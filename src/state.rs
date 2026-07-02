use crate::common::{HttpClient, JwtKeys};
use redis::aio::ConnectionManager;
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct AppState {
    // MySqlPool 内部是引用计数共享连接池，所以 AppState 可以 Clone 并分发给每个请求。
    pub db: MySqlPool,
    // Redis 不是核心依赖，用 Option 表示“可能不可用”；handler 里会自动跳过缓存。
    pub redis: Option<ConnectionManager>,
    // JwtKeys 保存签发和校验 token 所需的密钥。
    pub jwt: JwtKeys,
    // 统一 HTTP 客户端。
    pub http: HttpClient,
}
