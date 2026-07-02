use crate::common::{HttpClient, JwtKeys};
use crate::tencent_cloud::tencent_live_model::LiveUrlConfig;
use crate::tencent_cloud::tencent_live_signer::LiveCredential;
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
    // 腾讯云直播凭证。未配置时只禁用 live 接口，不影响核心业务启动。
    pub tencent_live_credential: Option<LiveCredential>,
    // 腾讯云直播推流/播放 URL 配置。未配置时只禁用 URL 生成接口。
    pub tencent_live_url_config: Option<LiveUrlConfig>,
}
