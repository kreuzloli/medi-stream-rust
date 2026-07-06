use anyhow::Context;
use medi_stream_rust::common::{HttpClient, JwtKeys};
use medi_stream_rust::config::Settings;
use medi_stream_rust::logging;
use medi_stream_rust::routes::router;
use medi_stream_rust::state::AppState;
use redis::aio::ConnectionManager;
use sqlx::mysql::MySqlPoolOptions;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// 启动服务：加载配置、初始化依赖并注册 HTTP 路由。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 读取项目根目录下的 .env。失败时忽略，方便线上继续使用系统环境变量。
    dotenvy::dotenv().ok();

    // 初始化日志。返回的 guard 必须保存在 main 生命周期内，否则文件日志可能来不及刷盘。
    let _log_guard = logging::init();

    // Settings 负责把环境变量收敛成一个配置结构体，避免在业务代码里到处读 env。
    let settings = Settings::from_env()?;

    // MySqlPool 是 SQLx 的连接池，Clone 成本很低，内部共享真实连接池。
    let db = MySqlPoolOptions::new()
        .max_connections(settings.mysql_max_connections)
        .connect(&settings.database_url)
        .await
        .context("connect mysql failed")?;

    // Redis 在这个项目里只做缓存。连接失败时不让服务启动失败，行为接近“缓存降级”。
    let redis = match redis::Client::open(settings.redis_url.as_str()) {
        Ok(client) => match ConnectionManager::new(client).await {
            Ok(manager) => Some(manager),
            Err(err) => {
                tracing::warn!(error = %err, "redis unavailable, continue without cache");
                None
            }
        },
        Err(err) => {
            tracing::warn!(error = %err, "invalid redis url, continue without cache");
            None
        }
    };
    let http = HttpClient::new(settings.http_timeout_seconds)?;
    // AppState 相当于 Spring Bean 容器里常用的共享依赖：数据库、Redis、JWT 工具。
    let state = AppState {
        db,
        redis,
        jwt: JwtKeys::from_settings(&settings)?,
        http,
        tencent_live_credential: settings.tencent_live_credential,
        tencent_live_url_config: settings.tencent_live_url_config,
        wechat_token: settings.wechat_token,
        wechat_app_id: settings.wechat_app_id,
        wechat_app_secret: settings.wechat_app_secret,
        wechat_encoding_aes_key: settings.wechat_encoding_aes_key,
        wechat_token_expire_seconds: settings.wechat_token_expire_seconds,
    };

    // Axum 的 Router 类似 Spring Controller 的路由注册表。
    let app = router(state).layer(TraceLayer::new_for_http());
    let listener = TcpListener::bind(&settings.server_addr).await?;
    tracing::info!(addr = %settings.server_addr, "medi-stream-rust started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// 等待终止信号，用于触发 Axum 优雅停机。
async fn shutdown_signal() {
    // 等待 Ctrl+C，然后让 axum 优雅停止监听新请求。
    let _ = tokio::signal::ctrl_c().await;
}
