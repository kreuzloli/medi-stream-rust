# medi-stream-admin-rust 管理后台服务设计

## 目标

在 `/Users/iris/Code/medi-stream-admin-rust` 新建独立 Rust 后台服务，复用 `medi-stream-rust` 的 Axum、SQLx、Redis、JWT、日志、错误响应和分页风格，只实现管理员账号、角色、权限及鉴权能力，不实现任何 `live_room` 业务。

## 项目边界

项目包含：

- 首个管理员初始化 CLI。
- 管理员登录、退出和当前管理员信息。
- 管理员查询、新增、修改、启停、重置密码和物理删除。
- 角色查询、新增、修改、启停和物理删除。
- 权限查询、新增、修改、启停和物理删除。
- 管理员多角色分配。
- 角色多权限分配。
- JWT、Redis Token 管理和接口权限校验。

项目不包含：

- 管理员公开注册接口。
- 普通用户、医院、科室、疾病、微信、腾讯云直播和直播间业务。
- 管理后台前端页面。
- 对共享数据库管理员表结构的重复建表或自动迁移。

## 技术方案

新建干净的 Rust 项目，只复制或重写参考项目中必要的基础设施，不通过复制整个业务项目再删减的方式构建。技术栈保持一致：

- Axum 0.7 提供 HTTP 服务。
- SQLx 0.8 连接共享 MySQL。
- Redis 保存有效 Token 和管理员 Token 索引。
- Argon2 保存和验证密码哈希。
- JSON Web Token 承载管理员身份、角色和权限。
- tracing 输出控制台和按日滚动文件日志。

不新增实现本需求不需要的依赖。

## 目录结构

```text
src/
├── admin/        管理员模型、Repository、Service、Handler
├── auth/         登录、退出、当前管理员和鉴权提取器
├── role/         角色 CRUD 和管理员角色分配
├── permission/   权限 CRUD 和角色权限分配
├── common/       JWT、Redis Token、分页、校验和常量
├── config.rs
├── error.rs
├── logging.rs
├── routes.rs
├── state.rs
├── lib.rs
└── main.rs
```

每个业务模块保持 Handler、Service、Repository、Model 分层。Handler 只处理 HTTP 参数和响应；Service 承担校验、权限和事务边界；Repository 只处理数据库读写。

## 配置

将参考项目当前 `.env` 原样复制到新项目，以保持共享数据库、Redis、JWT 和服务配置一致。`.env` 必须加入 `.gitignore`，不得提交。另生成只包含变量名和安全示例值的 `.env.example`。

后台服务只读取以下必要配置：

- `SERVER_ADDR`
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET_BASE64`
- `JWT_ISSUER`
- `JWT_TTL_SECONDS`
- `MYSQL_MAX_CONNECTIONS`

复制进 `.env` 的其他配置不读取，也不输出到日志。按照用户要求，实施时不修改复制后的 `SERVER_ADDR`；因此两个服务不能使用同一地址和端口同时启动。需要同时运行时，由部署方为后台服务单独覆盖 `SERVER_ADDR`。

所有依赖下载和构建命令通过交互式 zsh 执行 `proxy_on` 后运行。

## 数据模型

直接使用共享数据库中的：

- `administrator`
- `admin_role`
- `admin_permission`
- `administrator_role`
- `role_permission`

一个管理员允许拥有多个角色，一个角色允许拥有多个权限。关联分配使用事务执行“删除旧关联、插入新关联”，请求中的重复 ID 在写库前去重，并验证目标管理员、角色和权限存在。

`administrator.is_deleted` 保留以兼容现有共享表，但本项目不使用软删除。所有删除接口执行物理删除。

## 首个管理员初始化

项目提供一次性 CLI：

```bash
cargo run -- bootstrap-admin --username admin
```

初始化规则：

- 不开放管理员注册 HTTP 接口。
- 密码优先从安全终端输入；为部署自动化保留受控环境变量输入方式。
- 密码不得作为命令行参数，避免出现在 shell history 和进程列表。
- 密码使用 Argon2 哈希后写入数据库。
- 用户名已存在时明确失败，不覆盖原管理员。
- 初始化成功日志只记录管理员 ID 和用户名，不记录密码或哈希。

## 登录与 Token

登录流程：

1. 按用户名查询管理员。
2. 验证记录存在、`status = 1` 且 `is_deleted = 0`。
3. 使用 Argon2 验证密码。
4. 查询启用角色和启用权限。
5. 生成包含管理员 ID、角色编码、权限编码的 JWT。
6. 将 Token 写入 Redis，并建立管理员 ID 到 Token 的索引。
7. 更新 `last_login_at`。

每次受保护请求同时验证 JWT 签名、有效期和 Redis 中的 Token。退出时删除当前 Token。管理员被停用、删除或重置密码时，删除该管理员的全部有效 Token。

## 权限控制

首版使用以下权限编码：

- `ADMIN_VIEW`：查看管理员。
- `ADMIN_MANAGE`：新增、修改、启停、重置密码和删除管理员，分配管理员角色。
- `ROLE_VIEW`：查看角色。
- `ROLE_MANAGE`：新增、修改、启停和删除角色，分配角色权限。
- `PERMISSION_VIEW`：查看权限。
- `PERMISSION_MANAGE`：新增、修改、启停和删除权限。

登录、健康检查和初始化 CLI 不经过 HTTP 权限校验。`/auth/me` 和 `/auth/logout` 只要求有效管理员 Token。其余接口在 Handler 入口通过统一权限提取器校验权限编码。

首个管理员由 CLI 创建后，如果数据库尚无角色和权限，CLI 同时初始化一组系统管理员角色及上述权限，并把该管理员绑定到系统管理员角色，确保可以进入后台继续管理。

## HTTP 接口

认证接口：

- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/me`

管理员接口：

- `GET /admins`
- `GET /admins/:id`
- `POST /admins`
- `PUT /admins/:id`
- `PUT /admins/:id/status`
- `PUT /admins/:id/password`
- `PUT /admins/:id/roles`
- `DELETE /admins/:id`

角色接口：

- `GET /roles`
- `GET /roles/:id`
- `POST /roles`
- `PUT /roles/:id`
- `PUT /roles/:id/status`
- `PUT /roles/:id/permissions`
- `DELETE /roles/:id`

权限接口：

- `GET /permissions`
- `GET /permissions/:id`
- `POST /permissions`
- `PUT /permissions/:id`
- `PUT /permissions/:id/status`
- `DELETE /permissions/:id`

分页接口沿用参考项目的 `Page<T>` 响应结构。请求和响应使用 camelCase JSON。

## 删除行为

所有删除均为物理删除：

- 删除管理员前清理其 Redis Token，再执行 `DELETE FROM administrator`；外键级联删除 `administrator_role`。
- 删除角色时执行 `DELETE FROM admin_role`；外键级联删除管理员角色和角色权限关系。
- 删除权限时执行 `DELETE FROM admin_permission`；外键级联删除角色权限关系。

删除不可恢复。系统禁止当前管理员删除自己，避免当前会话在响应前失去主体；如需删除当前管理员，应由另一名具有 `ADMIN_MANAGE` 权限的管理员操作。

## 校验与错误处理

- 用户名、角色编码、角色名称、权限编码和权限名称不能为空。
- 用户名、角色编码、角色名称和权限编码保持数据库唯一约束，并在 Service 返回可读错误。
- 状态只允许 `0` 或 `1`。
- 新密码执行最小长度校验，且不记录明文。
- 登录失败统一返回“用户名或密码错误”，避免泄露账号是否存在。
- 找不到目标记录返回 404；请求非法返回 400；无有效 Token 返回 401；权限不足返回 403。
- 数据库、Redis 和 JWT 的原始错误保留为内部原因，对外返回统一错误结构。

## 日志

记录：

- 登录成功、失败和退出。
- 管理员、角色、权限的新增、修改、启停和删除。
- 角色和权限分配。
- Token 清理结果。

写操作日志包含操作者管理员 ID、目标记录 ID 和动作。不得记录密码、密码哈希、完整 Token、JWT 密钥或数据库连接串。

## 测试与验证

- 使用测试先行覆盖密码校验、状态校验、权限判定和请求模型。
- 覆盖管理员、角色、权限 Service 的新增、修改、物理删除和分配规则。
- 覆盖登录、Redis Token 校验、退出和批量 Token 失效。
- 覆盖重复用户名、重复角色/权限编码、目标不存在和自删除拒绝。
- 覆盖 CLI 首次初始化、重复初始化和系统角色权限初始化。
- 执行针对改动文件的格式化检查、`cargo check` 和全量 `cargo test`。
- 使用共享库的测试环境验证 SQLx 查询与已有表结构一致。

## 交付物

- 可独立构建和启动的 `/Users/iris/Code/medi-stream-admin-rust` Git 项目。
- 源代码、测试、`.gitignore`、`.env.example`、README 和启动说明。
- 从参考项目复制但不提交的 `.env`。
- 首个管理员初始化命令和管理员 API 示例。
