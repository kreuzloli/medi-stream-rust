# medi-stream-rust 用户 Token 认证过滤器设计

## 目标

把当前散落在 Handler 中的 Bearer Token 解析和 JWT 校验迁移到统一的 Axum 中间件，并与
`medi-stream-admin-rust` 保持相同的认证数据流：过滤器完成认证，Handler 只读取已经认证的当前用户。

本次只调整认证职责的位置，不修改路由路径、HTTP 方法、DTO、数据库结构和业务权限模型。

## 当前问题

- `/auth/me`、`/auth/logout` 和 `/account/**` Handler 分别读取 `HeaderMap` 并调用
  `JwtKeys::require_headers`，存在重复逻辑。
- JWT 校验成功后没有统一检查 Redis 中的 Token 是否仍然有效，导致注销只删除缓存，但已签发 JWT
  在过期前仍可能继续访问接口。
- Handler 同时承担 HTTP 业务处理和认证解析，新增受保护接口时容易遗漏 Token 校验。

## 方案比较

### 方案一：只保护当前已经要求 Token 的接口

把 `/auth/me`、`/auth/logout` 和 `/account/**` 组合成受保护 Router，统一挂载认证中间件；其他接口
保持当前访问边界不变。

优点是改动最小，不会让现有公开调用突然返回 401。缺点是当前没有认证要求的医院写接口仍保持现状，
如果后续需要限制，应作为独立的权限规则变更处理。

### 方案二：除登录、注册和回调外全部保护

所有医院、目录、腾讯云和微信主动操作接口都要求用户 Token。

安全边界更严格，但会改变现有 API 合同，前端和第三方回调可能需要同步调整，不属于单纯的认证代码迁移。

### 方案三：全局可选认证中间件

所有请求都经过中间件；有 Token 时解析并注入身份，没有 Token 时继续执行，再由 Handler 决定是否必须登录。

路由组装简单，但仍需要各 Handler 声明或检查认证要求，无法彻底解决认证遗漏问题。

## 选定方案

采用方案一，保持现有接口访问边界：

- 公开路由：`POST /auth/login`、`POST /auth/register`。
- 受保护认证路由：`GET /auth/me`、`GET /auth/logout`。
- 受保护账号路由：全部 `/account/**` 路由。
- 其余 hospital、catalog、Tencent Cloud 和 WeChat 路由保持当前公开行为。

## 组件与数据流

### CurrentUser

新增当前用户提取器，内部保存已验证的 `Claims`。提取器只从 request extensions 读取认证结果；如果中间件
未写入身份，则返回 401，不再自行解析请求头。

### authenticate_user

认证中间件按以下顺序处理请求：

1. 从 `Authorization` 读取严格的 `Bearer <token>`。
2. 校验 JWT 签名、算法、签发者和有效期，得到 `Claims`。
3. 检查 Redis Token 缓存是否存在；不存在时返回 401。
4. 将 `CurrentUser(Claims)` 和 Token 原文写入 request extensions。
5. 调用后续 Handler。

Token 原文只供注销接口删除当前会话使用，不写入日志和响应。

### 路由组装

`auth` 模块拆分为 `public_routes()` 和 `routes()`；`account::routes()` 全部属于受保护路由。
顶层 `routes.rs` 合并受保护 Router 后，通过 `route_layer(from_fn_with_state(...))` 统一挂载认证中间件，
再与其他公开 Router 合并。

### Handler

- `me` 和账号 Handler 通过 `CurrentUser` 获取 Claims。
- `logout` 同时提取 `CurrentUser` 和认证中间件保存的 Token 原文，删除当前 Token 缓存。
- 删除 Handler 中的 `HeaderMap`、`require_headers` 和 `get_token_from_headers` 调用。
- `JwtKeys` 保留签发和底层解码能力，删除只供 Handler 使用的 Header 解析方法。

## Redis 与错误处理

- Redis 中存在 Token：继续请求。
- Redis 中不存在 Token：返回 401，表示 Token 已失效。
- Redis 未配置或访问失败：返回 500，采用 fail-closed 行为，避免绕过注销失效机制。
- Header 缺失、Bearer 格式错误、JWT 无效或过期：返回 401。
- 不吞掉 JWT 或 Redis 错误原因，继续使用项目现有 `AppError` 映射。

## 验证

先增加路由级回归测试并确认旧实现下失败，再实施过滤器：

- 登录和注册不带 Token 时仍能进入对应 Handler，不被认证层提前返回 401。
- `/auth/me` 和 `/account` 不带 Token 时返回 401。
- 有效 JWT 但 Redis 不可用时，受保护路由返回 500。
- `CurrentUser` 能读取中间件注入的 Claims，缺少注入时返回 401。
- 注销继续删除当前 Token，且 Handler 不再解析 Header。

最终执行格式检查、完整编译和测试，确认没有扩大改动范围。

## 非目标

- 不增加角色或权限校验。
- 不调整医院、目录、腾讯云或微信接口的公开范围。
- 不修改 Token TTL、JWT Claims、Redis key 格式或登录响应。
- 不修改当前工作区已有的 `db/medi.sql` 和 `src/common/constants.rs` 未提交改动。
