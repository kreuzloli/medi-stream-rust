# medi-stream-rust

`medi-stream-rust` 是原 Spring Boot `medi-stream` 服务的 Rust + Axum 改写版本。项目当前实现了账号认证、账号资料与登录方式绑定、医院与科室疾病目录、腾讯云直播 URL 生成、直播状态查询、微信回调与 OAuth 授权等后端能力。

## 技术栈

- Axum 0.7：HTTP 路由和 handler。
- Tokio：异步运行时。
- SQLx：MySQL 数据访问。
- Redis：账号、token、目录、微信 access token 等缓存。
- jsonwebtoken：JWT 签发和校验。
- reqwest：腾讯云、微信等外部 HTTP 调用。
- tracing / tracing-appender：控制台日志和按天滚动文件日志。

## 目录结构

```text
.
├── db/                         # MySQL 建表脚本
├── docs/                       # 业务说明和历史文档
├── scripts/                    # 构建脚本
├── src/
│   ├── account/                # 账号资料、登录方式绑定/解绑
│   ├── auth/                   # 注册、登录、登出、当前用户
│   ├── common/                 # JWT、缓存、分页、校验、HTTP 客户端等通用能力
│   ├── hospital/               # 医院 CRUD、科室疾病目录
│   ├── live/                   # 直播领域模型和服务
│   ├── tencent_cloud/          # 腾讯云直播签名、URL 生成、状态查询
│   ├── wechat/                 # 微信回调、access token、OAuth
│   ├── config.rs               # 环境变量配置读取
│   ├── error.rs                # 统一错误类型和 HTTP 响应转换
│   ├── logging.rs              # 日志初始化
│   ├── main.rs                 # 程序入口
│   ├── routes.rs               # 全局路由表
│   └── state.rs                # AppState 共享依赖
├── tests/                      # 单元/集成测试
├── Cargo.toml
├── PROJECT_GUIDE.md            # 面向 Rust 初学者的项目导览
└── README.md
```

## 环境要求

- Rust stable toolchain。
- MySQL，数据库结构参考 `db/medi.sql`。
- Redis 可选；连接失败时服务会继续启动，但缓存和验证码相关能力会受影响。
- 如需腾讯云直播或微信能力，需要配置对应平台参数。

## 快速启动

复制环境变量模板：

```bash
cp .env.example .env
```

至少填写以下配置：

```env
SERVER_ADDR=0.0.0.0:8080
DATABASE_URL=mysql://user:password@127.0.0.1:3306/live
JWT_SECRET_BASE64=replace-with-base64-encoded-32-byte-secret
```

生成一个可用的 JWT secret 示例：

```bash
openssl rand -base64 32
```

启动服务：

```bash
cargo run
```

默认监听：

```text
http://127.0.0.1:8080
```

查看更详细日志：

```bash
RUST_LOG=debug cargo run
```

## 环境变量

| 变量 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `SERVER_ADDR` | 否 | `0.0.0.0:8080` | HTTP 服务监听地址。 |
| `DATABASE_URL` | 是 | 无 | MySQL 连接串。 |
| `REDIS_URL` | 否 | 项目默认 Redis 地址 | Redis 连接串；不可用时会降级。 |
| `JWT_SECRET_BASE64` | 是 | 无 | Base64 编码的 JWT 签名密钥。 |
| `JWT_ISSUER` | 否 | `medistream` | JWT issuer。 |
| `JWT_TTL_SECONDS` | 否 | `7200` | JWT 有效期秒数。 |
| `MYSQL_MAX_CONNECTIONS` | 否 | `10` | MySQL 连接池最大连接数。 |
| `HTTP_TIMEOUT_SECONDS` | 否 | `10` | 外部 HTTP 请求超时秒数。 |
| `TENCENT_LIVE_SECRET_ID` | 按功能 | 空 | 腾讯云 OpenAPI SecretId。 |
| `TENCENT_LIVE_SECRET_KEY` | 按功能 | 空 | 腾讯云 OpenAPI SecretKey。 |
| `TENCENT_LIVE_APP_NAME` | 按功能 | 空 | 腾讯云直播 appName。 |
| `TENCENT_LIVE_PUSH_DOMAIN` | 按功能 | 空 | 推流域名。 |
| `TENCENT_LIVE_PLAY_DOMAIN` | 按功能 | 空 | 播放域名。 |
| `TENCENT_LIVE_PUSH_KEY` | 按功能 | 空 | 推流防盗链 key。 |
| `TENCENT_LIVE_PLAY_KEY` | 按功能 | 空 | 播放防盗链 key。 |
| `TENCENT_LIVE_DEFAULT_TTL_SECONDS` | 否 | `86400` | 直播 URL 默认过期时间。 |
| `WECHAT_APP_ID` | 按功能 | 空 | 微信公众号/开放平台 AppID。 |
| `WECHAT_APP_SECRET` | 按功能 | 空 | 微信 AppSecret。 |
| `WECHAT_ENCODING_AES_KEY` | 按功能 | 空 | 微信消息加解密 key。 |
| `WECHAT_TOKEN` | 按功能 | 空 | 微信服务器回调校验 token。 |
| `WECHAT_ACCESS_TOKEN_EXPIRE_SECONDS` | 否 | `7200` | 微信 access token 缓存时间。 |
| `WEB_BASE_URL` | 否 | 项目默认前端地址 | OAuth 完成后回跳的前端地址。 |
| `WECHAT_OAUTH_CALLBACK_BASE_URL` | 按功能 | 空 | 微信 OAuth 回调的后端基础地址。 |

腾讯云直播 URL 生成相关的 `TENCENT_LIVE_APP_NAME`、推流域名、播放域名、推流 key、播放 key 必须同时配置；腾讯云 OpenAPI 的 `SECRET_ID` 和 `SECRET_KEY` 也必须同时配置。

## API 路由

路由注册入口在 `src/routes.rs`。

### 认证

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/auth/register` | 注册账号，成功后返回 JWT。 |
| `POST` | `/auth/login` | 登录并返回 JWT。支持邮箱密码、手机号验证码、微信/GitHub 第三方标识。 |
| `GET` | `/auth/logout` | 注销当前 token。需要 `Authorization: Bearer <token>`。 |
| `GET` | `/auth/me` | 返回当前 JWT 中的用户身份和角色。需要 token。 |

登录示例：

```bash
curl -X POST http://127.0.0.1:8080/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "loginType": "EMAIL",
    "loginIdentifier": "doctor@example.com",
    "password": "secret-123456"
  }'
```

### 账号

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/account` | 查询当前 JWT 用户的账号详情，优先读取 Redis 缓存。 |
| `POST` | `/account/bind/login` | 为当前用户绑定新的登录方式。 |
| `DELETE` | `/account/unbind/:login_id` | 解绑当前用户的一条登录方式。 |

账号相关接口都需要：

```text
Authorization: Bearer <token>
```

### 科室疾病目录

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/catalog/departments?includeDiseases=false` | 查询科室列表，可选带出疾病列表。 |
| `GET` | `/catalog/departments/:dept_id/diseases` | 查询指定科室下的疾病列表。 |
| `GET` | `/catalog/full` | 查询完整科室疾病目录，并使用 Redis 缓存。 |

示例：

```bash
curl http://127.0.0.1:8080/catalog/full
```

### 医院

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/hospitals` | 分页查询医院，支持名称、编码、省市、状态等筛选。 |
| `POST` | `/hospitals` | 创建医院。 |
| `GET` | `/hospitals/:id` | 查询单个医院。 |
| `PUT` | `/hospitals/:id` | 更新医院。 |
| `DELETE` | `/hospitals/:id` | 删除医院。 |

分页查询示例：

```bash
curl 'http://127.0.0.1:8080/hospitals?page=1&size=10&hospitalName=北京'
```

### 直播与腾讯云

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/live/urls` | 生成推流 URL、播放 URL 和可选转码播放 URL。 |
| `POST` | `/live/stream-state` | 调用腾讯云 Live OpenAPI 查询直播流状态。 |

生成直播 URL 示例：

```bash
curl 'http://127.0.0.1:8080/live/urls?streamName=test-stream&ttlSeconds=3600&transcodeTemplate=hd'
```

`/live/urls` 返回字段包含 WebRTC/RTMP 推流地址、WebRTC/RTMP/FLV/HLS 播放地址，以及可选转码 FLV/HLS 地址。

查询直播状态示例：

```bash
curl -X POST http://127.0.0.1:8080/live/stream-state \
  -H 'Content-Type: application/json' \
  -d '{
    "AppName": "live",
    "DomainName": "live.example.com",
    "StreamName": "test-stream"
  }'
```

### 微信

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/wechat/callback` | 微信服务器回调签名校验。 |
| `GET` | `/wechat/reload-access-token` | 重新获取并缓存微信 access token。 |
| `GET` | `/wechat/oauth/authorize` | 构建并跳转微信 OAuth 授权地址。 |
| `GET` | `/wechat/oauth/callback` | 处理微信 OAuth 回调，成功后跳回前端。 |

微信 OAuth 授权入口示例：

```bash
curl 'http://127.0.0.1:8080/wechat/oauth/authorize?redirect=/wechat-live-play'
```

## 日志

项目同时输出控制台日志和按天滚动的文件日志。日志文件位置：

```text
logs/medi-stream-rust.log.YYYY-MM-DD
```

日志级别可通过 `RUST_LOG` 控制：

```bash
RUST_LOG=medi_stream_rust=debug,tower_http=info cargo run
```

## 构建

本地构建：

```bash
cargo build --release
```

Apple Silicon Mac 上构建 Linux amd64 产物可以使用现有 Docker 脚本：

```bash
./scripts/build-linux-amd64.sh
```

构建结果：

```text
target/linux-amd64/release/medi-stream-rust
```

## 测试与检查

常用检查命令：

```bash
cargo fmt --check
cargo check
cargo test
```

当前测试覆盖账号、目录、医院、直播 URL、腾讯云签名和通用校验等逻辑，测试文件位于 `tests/`。

## 部署提示

- 生产环境至少需要准备二进制文件、`.env`、数据库结构和启动脚本。
- `.env` 中必须提供 `DATABASE_URL` 和 `JWT_SECRET_BASE64`。
- Redis 不是启动硬依赖，但验证码、token 缓存、目录缓存、微信 access token 缓存等能力会依赖 Redis。
- 当前 CORS 使用 permissive 配置，方便本地联调；上线时建议改成明确的前端域名白名单。
- URL 中的特殊字符需要转义，例如密码里的 `@` 写成 `%40`，`#` 写成 `%23`。

## 相关文档

- `PROJECT_GUIDE.md`：面向 Rust 初学者的项目结构和 Java/Rust 对照说明。
- `docs/腾讯云子域名配置全流程.md`：腾讯云直播域名配置说明。
- `docs/medi-stream现存功能开发日志.md`：当前已实现功能的开发记录。
