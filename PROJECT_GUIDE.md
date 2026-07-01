# medi-stream-rust 项目说明

这是一份给 Rust 初学者看的项目导览。这个项目是把原来的 Java Spring Boot `medi-stream` 服务，单独改写成 Rust + Axum 版本。

## 技术栈

- Axum：Web 框架，对应 Spring MVC Controller / Route。
- Tokio：异步运行时，Axum、SQLx、Redis 都跑在它上面。
- SQLx：MySQL 访问库，对应 Java 里的 MyBatis-Plus / Mapper。
- Redis：缓存 `full_department` 和 `user_info:{id}`。
- jsonwebtoken：生成和校验 HS256 JWT。
- Serde：JSON 序列化和反序列化，对应 Jackson。
- dotenvy：本地启动时读取 `.env`。
- tracing / tracing-subscriber / tracing-appender：日志门面、日志格式化、文件滚动写入。

## 目录结构

```text
medi-stream-rust/
├── Cargo.toml              # Rust 项目配置和依赖，类似 Maven pom.xml
├── Cargo.lock              # 锁定依赖版本，应用项目建议提交
├── .env                    # 本地运行配置，包含数据库、Redis、JWT
├── .env.example            # 配置模板，不放真实密码时给别人参考
├── README.md               # 快速启动说明
├── PROJECT_GUIDE.md        # 当前这份项目导览
├── db/
│   └── medi.sql            # MySQL 建表脚本
├── src/
│   ├── main.rs             # 程序入口：加载配置、连接 MySQL/Redis、启动 Axum
│   ├── lib.rs              # 声明模块，让测试和 main 都能引用
│   ├── config.rs           # 读取环境变量，生成 Settings
│   ├── logging.rs          # 初始化控制台日志和按天滚动的文件日志
│   ├── state.rs            # 全局共享状态 AppState：数据库、Redis、JWT
│   ├── routes.rs           # 路由注册：URL -> handler 函数
│   ├── error.rs            # 统一错误类型和 HTTP 响应转换
│   ├── common/             # 通用结构，例如分页 Page
│   │   ├── mod.rs
│   │   └── page.rs
│   ├── auth/               # 登录和 JWT
│   │   ├── mod.rs
│   │   ├── handlers.rs     # /auth/login、/auth/me
│   │   └── jwt.rs          # JWT 签发和校验
│   ├── catalog/            # 科室和疾病目录
│   │   ├── mod.rs
│   │   ├── model.rs        # Department、Disease、DTO、Query
│   │   ├── repository.rs   # catalog 相关 SQL
│   │   ├── service.rs      # 目录组装和 N+1 查询规避
│   │   └── handlers.rs     # /catalog 下的接口
│   └── account/            # 账号 CRUD
│       ├── mod.rs
│       ├── model.rs        # UserInfo、分页查询参数
│       ├── repository.rs   # account 相关 SQL
│       ├── cache.rs        # user_info:{id} Redis 缓存
│       └── handlers.rs     # /account 下的接口
└── tests/
    └── catalog_tests.rs    # 当前只有疾病预览截断规则测试
```

每个业务目录里的 `mod.rs` 相当于这个业务包的入口。比如 `catalog/mod.rs` 里声明了：

```rust
pub mod handlers;
pub mod model;
pub mod repository;
pub mod service;
```

这样其他文件就可以通过 `crate::catalog::service` 或 `crate::catalog::model::DiseaseDto` 引用。

## Java 和 Rust 写法对照

| Java / Spring Boot | Rust / Axum |
| --- | --- |
| `@SpringBootApplication` | `src/main.rs` |
| `@RestController` | 各业务目录下的 `handlers.rs` 函数 |
| `@RequestMapping` / `@GetMapping` | `routes.rs` 的 `.route(...)` |
| `@RequestBody` | `Json<T>` |
| `@RequestParam` | `Query<T>` |
| `@PathVariable` | `Path<T>` |
| `@Autowired` / 构造器注入 | `State<AppState>` |
| MyBatis-Plus Mapper | 各业务目录下的 `repository.rs` |
| Jackson DTO | Serde `Serialize` / `Deserialize` |
| Filter 校验 JWT | handler 里调用 `state.jwt.require_headers(...)` |

## 请求入口怎么走

以 `GET /catalog/full` 为例：

1. `main.rs` 启动服务，并调用 `router(state)`。
2. `routes.rs` 注册 `/catalog/full` 到 `catalog::handlers::full_catalog`。
3. `catalog/handlers.rs` 里的 `full_catalog` 先尝试读 Redis 的 `full_department`。
4. 缓存命中则直接返回 JSON。
5. 缓存没有命中时，调用 `catalog::service::list_departments(&state, true)`。
6. `catalog/service.rs` 调用 `catalog/repository.rs` 查 MySQL。
7. 查出科室后，repository 一次性批量查疾病，避免 N+1 查询。
8. service 组装 `DepartmentWithDiseasesDto`，handler 补 `diseasesPreview`，再写回 Redis。

## 常见 Rust 类型说明

- `Result<T, AppError>`：表示成功返回 `T`，失败返回 `AppError`。函数里用 `?` 可以把错误直接返回给上层。
- `Option<T>`：表示可能有值，也可能没有。对应 Java 里可能为 `null` 的字段。
- `Vec<T>`：动态数组，对应 Java 的 `List<T>`。
- `String`：拥有所有权的字符串；`&str` 是借用的字符串切片。
- `&state`：不可变借用，只读不改。
- `&mut state`：可变借用，需要修改 Redis 连接管理器或缓存状态时使用。
- `async fn`：异步函数，调用数据库、Redis、网络服务时需要 `.await`。

## 配置说明

`.env` 当前按 Java 项目的 `application.properties` 补齐：

```text
SERVER_ADDR=0.0.0.0:8080
DATABASE_URL=mysql://...
REDIS_URL=redis://...
JWT_SECRET_BASE64=...
JWT_ISSUER=medistream
JWT_TTL_SECONDS=7200
MYSQL_MAX_CONNECTIONS=10
```

注意：URL 里的特殊字符必须转义。例如密码里的 `@` 要写成 `%40`，`#` 要写成 `%23`，否则连接串会被解析错。

## 本地启动

```bash
cd /Users/iris/Code/medi-stream-rust
cargo run
```

如果要看更详细日志：

```bash
RUST_LOG=debug cargo run
```

## 日志

项目使用 Rust `tracing` 生态，作用接近 Java 里的 `slf4j + log4j2`：

- `tracing`：代码里打日志，例如 `tracing::info!`、`tracing::warn!`。
- `tracing-subscriber`：决定日志输出格式和级别。
- `tracing-appender`：把日志写入文件，并支持滚动。
- `tower_http::TraceLayer`：记录 HTTP 请求日志。

当前日志会同时输出到控制台和文件。文件位置：

```text
logs/medi-stream-rust.log.YYYY-MM-DD
```

默认级别是 `info`。可以通过 `RUST_LOG` 调整：

```bash
RUST_LOG=debug cargo run
RUST_LOG=medi_stream_rust=debug,tower_http=info cargo run
```

## 简单验证

登录：

```bash
curl -X POST http://127.0.0.1:8080/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"loginType":"EMAIL","loginIdentifier":"doctor@example.com","password":"secret-123456"}'
```

查全量目录：

```bash
curl http://127.0.0.1:8080/catalog/full
```

访问账号接口时，把登录接口返回的 token 放进请求头：

```bash
curl http://127.0.0.1:8080/account?page=1\&size=10 \
  -H 'Authorization: Bearer <token>'
```

## 当前实现边界

- 登录已改为数据库账号认证：邮箱走密码，手机号走验证码，第三方登录走 `thirdPartyUnionId`。
- `/account` 需要 `user_info` 表，建表脚本已经放在 `db/medi.sql`。
- Redis 只做缓存，连接失败时服务仍会启动，只是缓存不可用。
- CORS 当前是 `permissive`，方便本地前端调试；上线时建议改为固定域名白名单。
