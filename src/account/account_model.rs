use crate::common::constants::account::{
    LOGIN_TYPE_EMAIL, LOGIN_TYPE_GITHUB, LOGIN_TYPE_PHONE, LOGIN_TYPE_WECHAT,
};
use chrono::NaiveDateTime;
use serde::{de, Deserialize, Deserializer, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoginType {
    Email,
    Phone,
    Wechat,
    Github,
}

impl LoginType {
    // 数据库存的是大写字符串，统一从这里转换，避免 SQL 里散落硬编码。
    /// 返回数据库里保存的登录方式字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            LoginType::Email => LOGIN_TYPE_EMAIL,
            LoginType::Phone => LOGIN_TYPE_PHONE,
            LoginType::Wechat => LOGIN_TYPE_WECHAT,
            LoginType::Github => LOGIN_TYPE_GITHUB,
        }
    }

    // 只有邮箱注册需要本地密码；手机、微信、GitHub 不在本表保存密码。
    /// 判断当前登录方式是否需要本地密码哈希。
    pub fn needs_local_password(self) -> bool {
        matches!(self, LoginType::Email)
    }

    /// 把请求里的登录方式字符串转换成枚举值。
    fn from_request_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_uppercase().as_str() {
            LOGIN_TYPE_EMAIL => Some(LoginType::Email),
            LOGIN_TYPE_PHONE => Some(LoginType::Phone),
            LOGIN_TYPE_WECHAT => Some(LoginType::Wechat),
            LOGIN_TYPE_GITHUB => Some(LoginType::Github),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for LoginType {
    /// 自定义反序列化，拒绝不支持的 loginType。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        LoginType::from_request_value(&value).ok_or_else(|| {
            de::Error::custom(format!(
                "loginType只支持EMAIL、PHONE、WECHAT、GITHUB，当前值: {value}"
            ))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Option<u64>,
    pub user_code: Option<String>,
    pub real_name: String,
    pub nickname: Option<String>,
    pub hospital_id: Option<u64>,
    pub dept_id: Option<u64>,
    pub identity_type: Option<String>,
    pub doctor_cert_no: Option<String>,
    pub id_card_no: Option<String>,

    /// 用户联系电话，仅作为资料字段，不自动创建 PHONE 登录方式。
    pub mobile: Option<String>,

    /// 用户头像对应的 file_object.id。
    pub header_id: Option<u64>,

    pub status: i32,
    pub version: i32,
    pub is_deleted: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginAccount {
    // 登录账户是可解绑的绑定关系，同一个用户可以有邮箱、手机、微信、GitHub 等多种入口。
    pub id: u64,
    pub user_id: u64,
    pub login_type: String,
    pub login_identifier: String,
    pub third_party_union_id: Option<String>,
    pub is_verified: i32,
    pub last_login_at: Option<NaiveDateTime>,
    pub status: i32,
    pub is_deleted: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetail {
    pub profile: UserProfile,
    pub login_accounts: Vec<UserLoginAccount>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResp {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountReq {
    // 注册时可以一次性创建多个登录方式，最终每个登录方式对应一条 user_login_account。
    pub user_code: Option<String>,
    pub real_name: String,
    pub nickname: Option<String>,
    pub mobile: Option<String>,
    pub header_id: Option<u64>,
    pub hospital_id: Option<u64>,
    pub dept_id: Option<u64>,
    pub identity_type: Option<String>,
    pub doctor_cert_no: Option<String>,
    pub id_card_no: Option<String>,
    pub login_type: Option<LoginType>,
    pub login_identifier: Option<String>,
    pub password: Option<String>,
    pub third_party_union_id: Option<String>,
    pub is_verified: Option<i32>,
    pub status: Option<i32>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub verification_code: Option<String>,
    #[serde(default)]
    pub login_accounts: Vec<CreateLoginAccountReq>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserProfileReq {
    // 更新资料不允许顺手改登录标识或密码，避免把账号绑定和用户资料混在一个接口里。
    pub user_code: Option<String>,
    pub real_name: String,
    pub nickname: Option<String>,
    pub mobile: Option<String>,
    pub header_id: Option<u64>,
    pub hospital_id: Option<u64>,
    pub dept_id: Option<u64>,
    pub identity_type: Option<String>,
    pub doctor_cert_no: Option<String>,
    pub id_card_no: Option<String>,
    pub status: Option<i32>,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoginAccountReq {
    pub login_type: LoginType,
    pub login_identifier: String,
    pub password: Option<String>,
    pub third_party_union_id: Option<String>,
    pub is_verified: Option<i32>,
    pub status: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPageQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub user_code: Option<String>,
    pub real_name: Option<String>,
    pub hospital_id: Option<u64>,
    pub dept_id: Option<u64>,
    pub identity_type: Option<String>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthLoginAccount {
    // 仅认证流程内部使用，不作为接口响应返回，避免泄露 password_hash。
    pub user_id: u64,
    pub login_identifier: String,
    pub password_hash: Option<String>,
    pub third_party_union_id: Option<String>,
    pub is_verified: i32,
}

#[derive(Debug, Clone)]
pub struct LoginAccountForSave {
    pub login_type: LoginType,
    pub login_identifier: String,
    pub password_hash: Option<String>,
    pub third_party_union_id: Option<String>,
    pub is_verified: i32,
    pub status: i32,
}
