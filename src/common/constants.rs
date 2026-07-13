/// 项目公共常量。
///
/// 这里只放“跨模块复用”或“容易写错”的固定值。
/// 不建议把所有中文错误消息都搬进来，否则常量文件会变成错误消息仓库。
pub mod env {
    /// 服务监听地址，例如 0.0.0.0:8080。
    pub const SERVER_ADDR: &str = "SERVER_ADDR";
    pub const DEFAULT_SERVER_ADDR: &str = "0.0.0.0:8080";

    /// MySQL 连接串。
    pub const DATABASE_URL: &str = "DATABASE_URL";

    /// Redis 连接串。
    pub const REDIS_URL: &str = "REDIS_URL";
    pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379/0";

    /// JWT 配置。
    pub const JWT_SECRET_BASE64: &str = "JWT_SECRET_BASE64";
    pub const JWT_ISSUER: &str = "JWT_ISSUER";
    pub const DEFAULT_JWT_ISSUER: &str = "medistream";
    pub const JWT_TTL_SECONDS: &str = "JWT_TTL_SECONDS";
    pub const DEFAULT_JWT_TTL_SECONDS: &str = "7200";

    /// MySQL 连接池配置。
    pub const MYSQL_MAX_CONNECTIONS: &str = "MYSQL_MAX_CONNECTIONS";
    pub const DEFAULT_MYSQL_MAX_CONNECTIONS: &str = "10";

    /// 外部 HTTP API 请求超时时间，单位：秒。
    pub const HTTP_TIMEOUT_SECONDS: &str = "HTTP_TIMEOUT_SECONDS";
    pub const DEFAULT_HTTP_TIMEOUT_SECONDS: &str = "10";

    /// 腾讯云直播 API 凭证。
    pub const TENCENT_LIVE_SECRET_ID: &str = "TENCENT_LIVE_SECRET_ID";
    pub const TENCENT_LIVE_SECRET_KEY: &str = "TENCENT_LIVE_SECRET_KEY";

    /// 腾讯云直播推流/播放 URL 配置。
    pub const TENCENT_LIVE_APP_NAME: &str = "TENCENT_LIVE_APP_NAME";
    pub const TENCENT_LIVE_PUSH_DOMAIN: &str = "TENCENT_LIVE_PUSH_DOMAIN";
    pub const TENCENT_LIVE_PLAY_DOMAIN: &str = "TENCENT_LIVE_PLAY_DOMAIN";
    pub const TENCENT_LIVE_PUSH_KEY: &str = "TENCENT_LIVE_PUSH_KEY";
    pub const TENCENT_LIVE_PLAY_KEY: &str = "TENCENT_LIVE_PLAY_KEY";
    pub const TENCENT_LIVE_DEFAULT_TTL_SECONDS: &str = "TENCENT_LIVE_DEFAULT_TTL_SECONDS";
    pub const DEFAULT_TENCENT_LIVE_DEFAULT_TTL_SECONDS: &str = "86400";

    /// 腾讯云 Web 播放器 License 配置，仅允许服务端读取。
    pub const TENCENT_LIVE_LICENSE_URL: &str = "TENCENT_LIVE_LICENSE_URL";
    pub const TENCENT_LIVE_LICENSE_KEY: &str = "TENCENT_LIVE_LICENSE_KEY";

    /// 微信服务号配置。
    pub const WECHAT_APP_ID: &str = "WECHAT_APP_ID";
    pub const WECHAT_APP_SECRET: &str = "WECHAT_APP_SECRET";
    pub const WECHAT_ACCESS_TOKEN_EXPIRE_SECONDS: &str = "WECHAT_ACCESS_TOKEN_EXPIRE_SECONDS";
    pub const DEFAULT_WECHAT_ACCESS_TOKEN_EXPIRE_SECONDS: i64 = 7200;
    pub const WECHAT_ENCODING_AES_KEY: &str = "WECHAT_ENCODING_AES_KEY";
    pub const WECHAT_TOKEN: &str = "WECHAT_TOKEN";

    /// 前端 H5 基础地址。
    pub const WEB_BASE_URL: &str = "WEB_BASE_URL";
    pub const DEFAULT_WEB_BASE_URL: &str = "http://127.0.0.1:3000";
    /// 微信 OAuth 回调基础地址。
    pub const WECHAT_OAUTH_CALLBACK_BASE_URL: &str = "WECHAT_OAUTH_CALLBACK_BASE_URL";
}

pub mod cache {
    /// 用户详情缓存 key 前缀。
    ///
    /// 完整 key 示例：
    /// account_detail:123
    pub const ACCOUNT_DETAIL_CACHE_PREFIX: &str = "account_detail:";

    /// token 缓存 key 前缀。
    ///
    /// 完整 key 示例：
    /// token:xxxxx.yyyyy.zzzzz
    pub const TOKEN_CACHE_PREFIX: &str = "token:";

    /// 登录验证码缓存 key 前缀。
    ///
    /// 完整 key 示例：
    /// login_verification_code:PHONE:13800000000
    pub const LOGIN_VERIFICATION_CODE_PREFIX: &str = "login_verification_code:";

    /// 账号相关缓存时间，单位：秒。
    pub const ACCOUNT_CACHE_SECONDS: u64 = 10 * 60;

    /// 微信 access_token 缓存 key 前缀。

    ///

    /// 完整 key 示例：

    /// wechat_access_token:wx123456

    pub const WECHAT_ACCESS_TOKEN_PREFIX: &str = "wechat_access_token:";

    /// 微信 access_token 缓存时间，单位：秒。
    ///
    /// 微信默认 expires_in 是 7200 秒。
    /// 这里少缓存一点，避免临界时间 token 已经过期。
    pub const WECHAT_ACCESS_TOKEN_CACHE_SECONDS: u64 = 7100;
}

pub mod auth {
    /// HTTP Authorization 头里的 Bearer 前缀。
    pub const BEARER_PREFIX: &str = "Bearer ";

    /// 普通用户角色。
    pub const ROLE_USER: &str = "USER";

    /// 当前项目 JWT 使用的算法名。
    ///
    /// 这个常量主要用于注释、日志或文档场景；
    /// 真正创建 Header / Validation 时仍然使用 jsonwebtoken::Algorithm::HS256。
    pub const JWT_ALGORITHM_HS256: &str = "HS256";
}

pub mod account {
    /// 医药行业相关身份。
    pub const IDENTITY_MEDICAL_WORKER: &str = "MEDICAL_WORKER";

    /// 非医药行业相关身份。
    pub const IDENTITY_NON_MEDICAL_WORKER: &str = "NON_MEDICAL_WORKER";

    /// 默认登录账号绑定数量上限。
    ///
    /// 当前支持 EMAIL / PHONE / WECHAT / GITHUB，最多 4 种。
    pub const MAX_LOGIN_ACCOUNT_COUNT: usize = 4;

    /// 登录方式字符串，和数据库 login_type 字段保持一致。
    pub const LOGIN_TYPE_EMAIL: &str = "EMAIL";
    pub const LOGIN_TYPE_PHONE: &str = "PHONE";
    pub const LOGIN_TYPE_WECHAT: &str = "WECHAT";
    pub const LOGIN_TYPE_GITHUB: &str = "GITHUB";
}

pub mod status {
    /// 启用状态。
    pub const STATUS_ENABLED: i32 = 1;

    /// 禁用状态。
    pub const STATUS_DISABLED: i32 = 0;

    /// 未删除。
    pub const NOT_DELETED: i32 = 0;

    /// 已删除。
    pub const DELETED: i32 = 1;

    /// 默认版本号。
    pub const DEFAULT_VERSION: i32 = 0;

    /// 判断值是否为通用启用/禁用状态。
    pub fn is_enabled_or_disabled(value: i32) -> bool {
        matches!(value, STATUS_DISABLED | STATUS_ENABLED)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn is_enabled_or_disabled_accepts_only_zero_and_one() {
            assert!(is_enabled_or_disabled(STATUS_DISABLED));
            assert!(is_enabled_or_disabled(STATUS_ENABLED));
            assert!(!is_enabled_or_disabled(-1));
            assert!(!is_enabled_or_disabled(2));
        }
    }
}

pub mod page {
    /// 默认页码。
    pub const DEFAULT_PAGE: u64 = 1;

    /// 默认分页大小。
    pub const DEFAULT_PAGE_SIZE: u64 = 10;

    /// 最大分页大小，防止一次查太多。
    pub const MAX_PAGE_SIZE: u64 = 200;
}

pub mod http {
    /// 常用 HTTP header 名。
    ///
    /// reqwest / axum 自带的 header 常量也能用；
    /// 这里保留字符串常量主要是给腾讯云签名这种“必须精确拼接 header”的场景。
    pub const HEADER_AUTHORIZATION: &str = "Authorization";
    pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
    pub const HEADER_HOST: &str = "Host";

    /// 腾讯云专用 header。
    pub const HEADER_X_TC_ACTION: &str = "X-TC-Action";
    pub const HEADER_X_TC_VERSION: &str = "X-TC-Version";
    pub const HEADER_X_TC_TIMESTAMP: &str = "X-TC-Timestamp";
    pub const HEADER_X_TC_REGION: &str = "X-TC-Region";

    pub const CONTENT_TYPE_JSON_UTF8: &str = "application/json; charset=utf-8";
}

pub mod wechat {
    /// 微信服务号 API。
    pub const WECHAT_API_BASE_URL: &str = "https://api.weixin.qq.com";

    /// 获取 access_token。
    pub const WECHAT_ACCESS_TOKEN_PATH: &str = "/cgi-bin/token";

    /// grant_type 固定值。
    pub const WECHAT_CLIENT_CREDENTIAL: &str = "client_credential";

    /// 用于 HttpClient 日志里的 service 名。
    pub const WECHAT_SERVICE_NAME: &str = "wechat";

    /// 微信成功时一般没有 errcode，或者 errcode = 0。
    pub const WECHAT_SUCCESS_ERRCODE: i64 = 0;

    /// 微信网页 OAuth 授权地址。
    pub const WECHAT_OAUTH_AUTHORIZE_URL: &str =
        "https://open.weixin.qq.com/connect/oauth2/authorize";

    /// 微信网页授权 access_token。
    ///
    /// 注意：这个不是公众号全局 access_token。
    pub const WECHAT_OAUTH_ACCESS_TOKEN_PATH: &str = "/sns/oauth2/access_token";

    /// 微信 OAuth code 换 token 的 grant_type。
    pub const WECHAT_AUTHORIZATION_CODE: &str = "authorization_code";

    /// 静默授权，只拿 openId。
    pub const WECHAT_OAUTH_SCOPE_BASE: &str = "snsapi_base";

    /// 微信 OAuth 成功回跳路径。
    pub const WECHAT_OAUTH_CALLBACK_PATH: &str = "/wechat/oauth/callback";
}

pub mod tencent_cloud {
    /// 腾讯云直播 API endpoint。
    pub const TENCENT_LIVE_ENDPOINT: &str = "https://live.tencentcloudapi.com";

    /// 腾讯云直播 API host。
    pub const TENCENT_LIVE_HOST: &str = "live.tencentcloudapi.com";

    /// 腾讯云直播 service 名。
    pub const TENCENT_LIVE_SERVICE: &str = "live";

    /// 腾讯云直播 API 版本。
    pub const TENCENT_LIVE_VERSION: &str = "2018-08-01";

    /// 腾讯云 TC3 签名算法。
    pub const TENCENT_CLOUD_ALGORITHM: &str = "TC3-HMAC-SHA256";

    /// 用于 HttpClient 日志里的 service 名。
    pub const TENCENT_CLOUD_LIVE_SERVICE_NAME: &str = "tencent_cloud_live";

    /// 查询直播流状态。
    pub const ACTION_DESCRIBE_LIVE_STREAM_STATE: &str = "DescribeLiveStreamState";
}

pub mod route {
    /// 用户登录并获取访问令牌。
    pub const AUTH_LOGIN: &str = "/auth/login";
    /// 用户退出登录并使当前令牌失效。
    pub const AUTH_LOGOUT: &str = "/auth/logout";
    /// 获取当前已登录用户的信息。
    pub const AUTH_ME: &str = "/auth/me";
    /// 注册新的普通用户账号。
    pub const AUTH_REGISTER: &str = "/auth/register";

    /// 获取当前用户的账号详情。
    pub const ACCOUNT: &str = "/account";
    /// 为当前用户绑定新的登录方式。
    pub const ACCOUNT_BIND_LOGIN: &str = "/account/bind/login";
    /// 解除当前用户指定登录方式的绑定。
    pub const ACCOUNT_UNBIND: &str = "/account/unbind/:login_id";

    /// 查询全部启用的科室。
    pub const CATALOG_DEPARTMENTS: &str = "/catalog/departments";
    /// 查询指定科室下的疾病。
    pub const CATALOG_DEPARTMENT_DISEASES: &str = "/catalog/departments/:dept_id/diseases";
    /// 获取包含科室及疾病的完整目录。
    pub const CATALOG_FULL: &str = "/catalog/full";

    /// 分页查询医院或创建医院。
    pub const HOSPITALS: &str = "/hospitals";
    /// 查询、更新或删除指定医院。
    pub const HOSPITAL_BY_ID: &str = "/hospitals/:id";

    /// 根据直播配置生成腾讯云推流和播放地址。
    pub const LIVE_URLS: &str = "/live/urls";
    /// 查询腾讯云直播流的当前状态。
    pub const LIVE_STREAM_STATE: &str = "/live/stream-state";
    /// 获取前端播放器所需的腾讯云 License 配置。
    pub const LIVE_LICENSE: &str = "/live/license";

    /// 验证微信服务器回调请求的签名。
    pub const WECHAT_CALLBACK: &str = "/wechat/callback";

    /// 主动刷新微信公众号的全局 Access Token。
    pub const WECHAT_RELOAD_ACCESS_TOKEN: &str = "/wechat/reload-access-token";

    /// 微信网页授权入口。
    pub const WECHAT_OAUTH_AUTHORIZE: &str = "/wechat/oauth/authorize";

    /// 微信网页授权回调。
    pub const WECHAT_OAUTH_CALLBACK: &str = "/wechat/oauth/callback";
}
