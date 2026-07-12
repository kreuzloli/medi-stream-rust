# medi-stream-admin-rust 管理后台服务设计

## 目标

在 `/Users/iris/Code/medi-stream-admin-rust` 新建独立 Rust 后台服务，复用 `medi-stream-rust` 的 Axum、SQLx、Redis、JWT、日志、错误响应、分页风格和相关业务规则，为管理员提供账号权限、普通用户管理、医院目录管理、直播管理和必要的腾讯云直播能力。

## 项目边界

项目包含：

- 首个管理员初始化 CLI。
- 管理员登录、退出、当前管理员、管理员 CRUD、角色权限管理。
- 普通用户资料查询、封禁和解封。
- 医院、科室、疾病的后台管理。
- 文件对象、直播间和直播流的后台管理。
- 腾讯云推流/播放 URL 生成和直播流状态查询。
- JWT、Redis Token 和基于权限编码的接口鉴权。

项目不包含：

- 普通用户登录、注册、退出、密码和登录绑定管理。
- 微信回调、OAuth、Access Token 等微信能力。
- 用户端播放器 License 接口。
- 管理后台前端页面。
- 自动创建或迁移共享数据库表结构。

## 技术方案

新建干净项目，只复用参考项目中必要的基础设施和业务逻辑，不复制整个项目后再删除无关模块：

- Axum 0.7 提供 HTTP 服务。
- SQLx 0.8 连接共享 MySQL。
- Redis 保存有效 Token 和管理员 Token 索引。
- Argon2 保存和验证管理员密码哈希。
- JSON Web Token 承载管理员身份、角色和权限。
- reqwest 调用腾讯云 OpenAPI。
- tracing 输出控制台和按日滚动文件日志。

不新增本需求不需要的依赖。

## 目录结构

```text
src/
├── admin/          管理员 CRUD、启停、重置密码
├── auth/           管理员登录、退出、当前管理员和鉴权
├── role/           角色 CRUD 和管理员角色分配
├── permission/     权限 CRUD 和角色权限分配
├── user/           普通用户查询、封禁和解封
├── hospital/       医院 CRUD
├── catalog/        科室和疾病 CRUD
├── live/           文件对象、直播间和直播流管理
├── tencent_cloud/  URL 生成、签名和直播状态查询
├── common/         JWT、Redis Token、分页、校验和常量
├── config.rs
├── error.rs
├── logging.rs
├── routes.rs
├── state.rs
├── lib.rs
└── main.rs
```

业务模块保持 Handler、Service、Repository、Model 分层。Handler 只处理 HTTP 参数、管理员身份和响应；Service 承担校验、权限、日志和事务边界；Repository 只处理数据库读写。

## 配置与代理

将参考项目当前 `.env` 原样复制到新项目。`.env` 加入 `.gitignore`，不得提交；另生成只包含变量名和安全示例值的 `.env.example`。

后台服务读取：

- `SERVER_ADDR`
- `DATABASE_URL`
- `REDIS_URL`
- `JWT_SECRET_BASE64`
- `JWT_ISSUER`
- `JWT_TTL_SECONDS`
- `MYSQL_MAX_CONNECTIONS`
- `HTTP_TIMEOUT_SECONDS`
- 腾讯云直播 URL 和 OpenAPI 凭证相关变量

不读取微信和播放器 License 配置，也不输出任何密钥。按照用户要求不修改复制后的 `SERVER_ADDR`；两个服务同时运行时，由部署方为后台服务单独覆盖该变量。

所有依赖下载和构建命令通过交互式 zsh 执行 `proxy_on` 后运行。

## 管理员与 RBAC

直接使用共享数据库中的：

- `administrator`
- `admin_role`
- `admin_permission`
- `administrator_role`
- `role_permission`

管理员允许多个角色，角色允许多个权限。分配操作在事务中执行“删除旧关联、插入新关联”，请求 ID 去重后验证目标存在。

`administrator.is_deleted` 为共享表兼容字段，本项目不使用软删除。管理员、角色和权限删除都执行物理删除，并由外键级联删除关联数据。禁止当前管理员删除自己。

## 首个管理员初始化

提供：

```bash
cargo run -- bootstrap-admin --username admin
```

规则：

- 不开放管理员注册 HTTP 接口。
- 密码通过安全终端输入，部署自动化可使用受控环境变量。
- 密码不能作为命令行参数。
- 使用 Argon2 哈希，用户名已存在时失败，不覆盖原账号。
- 数据库没有系统角色和权限时，初始化内置权限及 `SUPER_ADMIN` 角色，并绑定首个管理员。
- 日志不记录密码或哈希。

## 登录与 Token

登录时验证管理员存在、`status = 1`、`is_deleted = 0` 和密码，加载启用角色及权限后生成 JWT。JWT 包含管理员 ID、角色编码和权限编码。Token 写入 Redis，并建立管理员 ID 到 Token 的索引。

受保护请求同时验证 JWT 和 Redis Token。退出删除当前 Token；管理员停用、删除或重置密码时删除其全部 Token。登录失败统一返回“用户名或密码错误”。

## 普通用户管理

普通用户模块只管理 `user_info`：

- 分页和详情。
- 按用户编码、姓名、医院、科室、身份类型和状态查询。
- 封禁：设置 `status = 0`。
- 解封：设置 `status = 1`。

不新增普通用户，不修改用户密码，不提供登录注册，不直接管理 `user_login_account`。封禁用户不删除资料和登录绑定；原用户服务应以 `user_info.status` 作为账号可用性判断依据。

## 医院、科室与疾病管理

医院、科室和疾病均提供分页/列表、详情、新增、修改、启停和物理删除。科室和疾病支持 `sort_no` 排序，疾病必须属于一个科室。

删除前检查引用：

- 医院被普通用户引用时拒绝删除。
- 科室被普通用户、疾病或直播间引用时拒绝删除。
- 疾病被直播间引用时拒绝删除。

引用存在时返回明确业务错误，不依赖外键错误作为正常控制流。

## 直播管理

后台项目管理 `file_object`、`live_room` 和 `live_room_stream`：

- 文件对象：分页、详情、新增和删除；被直播间封面引用时拒绝删除。
- 直播间：分页、详情、新增、修改、置顶、取消置顶、启停、封禁和物理删除。
- 直播流：分页/列表、详情、新增、修改、默认流、排序、启停和物理删除。

管理员创建直播间时，`owner_admin_id` 从当前 JWT 自动取得，`owner_user_id` 为空，请求不能伪造其他管理员 ID。后台可以查看普通用户创建的直播间。

修改直播间一般不改变房主；如使用独立的“变更房主”接口，则必须明确指定普通用户或管理员且严格二选一。管理员房主必须启用，普通用户房主必须未删除；科室和疾病均可为空，同时填写时疾病必须属于所选科室。

置顶排序为 `is_top DESC, id DESC`。删除直播间前先删除其直播流，使用事务保证一致性；文件对象不随直播间自动删除。

## 腾讯云直播

后台项目复用参考项目的腾讯云签名、URL 生成和直播状态查询逻辑：

- 生成推流和播放 URL。
- 查询指定域名、应用和流名称的直播状态。

不提供播放器 License 接口，不包含微信逻辑。腾讯云请求错误保留服务名、状态码和响应体用于排查，但不得记录 SecretId 或 SecretKey。

## 权限编码

- `ADMIN_VIEW`、`ADMIN_MANAGE`
- `ROLE_VIEW`、`ROLE_MANAGE`
- `PERMISSION_VIEW`、`PERMISSION_MANAGE`
- `USER_VIEW`、`USER_MANAGE`
- `HOSPITAL_VIEW`、`HOSPITAL_MANAGE`
- `CATALOG_VIEW`、`CATALOG_MANAGE`
- `LIVE_VIEW`、`LIVE_MANAGE`
- `TENCENT_LIVE_VIEW`、`TENCENT_LIVE_MANAGE`

登录和健康检查不校验权限；`/auth/me`、`/auth/logout` 只要求有效 Token；其他接口按查看或管理权限校验。

## HTTP 接口

认证：

- `POST /auth/login`
- `POST /auth/logout`
- `GET /auth/me`

管理员与 RBAC：

- `/admins`、`/admins/:id`：分页、详情、新增、修改、删除
- `/admins/:id/status`、`/admins/:id/password`、`/admins/:id/roles`
- `/roles`、`/roles/:id`：分页、详情、新增、修改、删除
- `/roles/:id/status`、`/roles/:id/permissions`
- `/permissions`、`/permissions/:id`：分页、详情、新增、修改、删除
- `/permissions/:id/status`

普通用户：

- `GET /users`
- `GET /users/:id`
- `PUT /users/:id/status`

医院和目录：

- `/hospitals`、`/hospitals/:id`
- `/departments`、`/departments/:id`、`/departments/:id/status`
- `/diseases`、`/diseases/:id`、`/diseases/:id/status`

直播：

- `/files`、`/files/:id`
- `/live-rooms`、`/live-rooms/:id`
- `/live-rooms/:id/status`、`/live-rooms/:id/top`、`/live-rooms/:id/owner`
- `/live-streams`、`/live-streams/:id`
- `/live-streams/:id/status`、`/live-streams/:id/default`

腾讯云：

- `GET /tencent-live/urls`
- `POST /tencent-live/stream-state`

分页接口沿用参考项目的 `Page<T>`，JSON 字段使用 camelCase。

## 校验、错误和日志

- 状态和布尔标记只允许 `0` 或 `1`；直播间状态额外允许封禁值 `2`。
- 唯一字段冲突返回可读业务错误。
- 记录不存在返回 404，请求非法返回 400，无 Token 返回 401，权限不足返回 403。
- 写日志包含操作者管理员 ID、目标 ID 和动作。
- 登录、退出、Token 清理、封禁解封、角色权限分配、直播置顶和状态变化都记录日志。
- 不记录密码、哈希、完整 Token、JWT 密钥、数据库连接串或腾讯云密钥。

## 测试与验证

- 测试先行覆盖密码、状态、权限、所有者互斥、科室疾病关系和请求模型。
- 覆盖管理员登录、Redis Token、物理删除、角色权限分配和自删除拒绝。
- 覆盖普通用户查询、封禁和解封，不出现登录注册路由。
- 覆盖医院目录引用删除保护。
- 覆盖管理员创建直播间自动取 JWT 管理员 ID、普通用户直播间查询、置顶排序和流管理。
- 覆盖腾讯云 URL 签名、请求校验和上游错误。
- 执行目标文件格式化检查、`cargo check` 和全量 `cargo test`。
- 使用共享库测试环境验证 SQLx 查询和物理删除级联关系。

## 交付物

- 独立 Git 项目 `/Users/iris/Code/medi-stream-admin-rust`。
- 源代码、测试、`.gitignore`、安全的 `.env.example`、README 和启动说明。
- 从参考项目复制但不提交的 `.env`。
- 首个管理员初始化命令、权限说明和 API 请求示例。
