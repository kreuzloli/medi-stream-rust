use crate::account::model::{
    AccountDetail, CreateAccountReq, CreateLoginAccountReq, LoginAccountForSave, LoginType,
    UpdateUserProfileReq,
};
use crate::account::{cache, repository};
use crate::auth::jwt::Claims;
use crate::error::AppError;
use crate::state::AppState;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand_core::OsRng;
use std::collections::HashSet;

const MEDICAL_WORKER: &str = "MEDICAL_WORKER";
const NON_MEDICAL_WORKER: &str = "NON_MEDICAL_WORKER";

pub fn validate_create_account_req(req: &CreateAccountReq) -> Result<(), AppError> {
    validate_profile_fields(
        &req.real_name,
        req.hospital_id,
        req.dept_id,
        &req.identity_type,
        req.status,
    )?;
    for login in account_login_reqs(req)? {
        validate_create_login_account_req(&login)?;
    }
    Ok(())
}

pub fn validate_update_user_profile_req(req: &UpdateUserProfileReq) -> Result<(), AppError> {
    validate_profile_fields(
        &req.real_name,
        req.hospital_id,
        req.dept_id,
        &req.identity_type,
        req.status,
    )
}

pub fn validate_create_login_account_req(req: &CreateLoginAccountReq) -> Result<(), AppError> {
    validate_login_identifier(req.login_identifier.as_str())?;
    // 只有本地登录方式需要密码；第三方登录的凭证由外部平台证明身份。
    if req.login_type.needs_local_password()
        && req
            .password
            .as_deref()
            .is_none_or(|password| password.trim().is_empty())
    {
        return Err(AppError::BadRequest("邮箱登录必须填写密码".to_string()));
    }
    validate_status(req.status)?;
    Ok(())
}

pub fn account_login_reqs(req: &CreateAccountReq) -> Result<Vec<CreateLoginAccountReq>, AppError> {
    let mut logins = if req.login_accounts.is_empty() {
        legacy_login_reqs(req)
    } else {
        req.login_accounts.clone()
    };

    if logins.is_empty() {
        return Err(AppError::BadRequest("至少绑定一个登录账户".to_string()));
    }
    if logins.len() > 4 {
        return Err(AppError::BadRequest("登录方式最多绑定4个".to_string()));
    }

    let mut login_types = HashSet::new();
    let mut login_keys = HashSet::new();
    for login in &mut logins {
        normalize_login_account(login);
        if !login_types.insert(login.login_type) {
            return Err(AppError::BadRequest("同一种登录方式不能重复".to_string()));
        }
        let login_key = format!(
            "{}:{}",
            login.login_type.as_str(),
            login.login_identifier.trim()
        );
        if !login_keys.insert(login_key) {
            return Err(AppError::BadRequest("登录账户不能重复".to_string()));
        }
    }

    Ok(logins)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    // Argon2 输出里包含算法参数和 salt，保存这一串即可支持后续 verify。
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Internal(err.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn login_password_hash(req: &CreateLoginAccountReq) -> Result<Option<String>, AppError> {
    if req.login_type.needs_local_password() {
        let password = req
            .password
            .as_deref()
            .ok_or_else(|| AppError::BadRequest("邮箱登录必须填写密码".to_string()))?;
        return hash_password(password).map(Some);
    }
    Ok(None)
}

pub fn account_token_subject(account: &AccountDetail) -> String {
    account
        .login_accounts
        .first()
        .map(|login| login.login_identifier.clone())
        .unwrap_or_else(|| account.profile.real_name.clone())
}

pub fn require_claim_user_id(claims: &Claims) -> Result<u64, AppError> {
    claims
        .uid
        .ok_or_else(|| AppError::Unauthorized("Token missing user id".to_string()))
}

pub async fn create_account(
    state: &mut AppState,
    req: CreateAccountReq,
) -> Result<crate::account::model::AccountDetail, AppError> {
    validate_create_account_req(&req)?;
    let login_accounts = account_login_reqs(&req)?
        .into_iter()
        .map(|login| login_account_for_save(&login))
        .collect::<Result<Vec<_>, _>>()?;
    // 用户资料和初始登录方式必须一起成功，避免产生无登录入口的用户资料。
    let id = repository::insert_account_with_logins(&state.db, &req, &login_accounts).await?;
    let account = repository::find_account_detail_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("account not found".to_string()))?;
    cache::cache_account(state, &account).await?;
    Ok(account)
}

pub async fn update_profile(
    state: &mut AppState,
    id: u64,
    req: UpdateUserProfileReq,
) -> Result<bool, AppError> {
    validate_update_user_profile_req(&req)?;
    let ok = repository::update_user_profile(&state.db, id, &req).await?;
    cache::delete_account_cache(state, id).await?;
    Ok(ok)
}

pub async fn add_login_account(
    state: &mut AppState,
    user_id: u64,
    req: CreateLoginAccountReq,
) -> Result<crate::account::model::UserLoginAccount, AppError> {
    validate_create_login_account_req(&req)?;
    // 只有邮箱会生成 password_hash；手机、微信、GitHub 的 password_hash 保持为空。
    let login = login_account_for_save(&req)?;
    let login_id = repository::insert_login_account(&state.db, user_id, &login).await?;
    cache::delete_account_cache(state, user_id).await?;
    repository::find_login_account_by_id(&state.db, user_id, login_id)
        .await?
        .ok_or_else(|| AppError::NotFound("login account not found".to_string()))
}

pub async fn delete_account(state: &mut AppState, id: u64) -> Result<bool, AppError> {
    let ok = repository::logical_delete_user(&state.db, id).await?;
    cache::delete_account_cache(state, id).await?;
    Ok(ok)
}

pub async fn unbind_login_account(
    state: &mut AppState,
    user_id: u64,
    login_id: u64,
) -> Result<bool, AppError> {
    let ok = repository::delete_login_account(&state.db, user_id, login_id).await?;
    cache::delete_account_cache(state, user_id).await?;
    Ok(ok)
}

fn legacy_login_reqs(req: &CreateAccountReq) -> Vec<CreateLoginAccountReq> {
    let mut logins = Vec::new();
    if let Some(email) = req
        .email
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        logins.push(CreateLoginAccountReq {
            login_type: LoginType::Email,
            login_identifier: email.trim().to_string(),
            password: req.password.clone(),
            third_party_union_id: None,
            is_verified: req.is_verified,
            status: req.status,
        });
    }
    if let Some(phone) = req
        .phone
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        logins.push(CreateLoginAccountReq {
            login_type: LoginType::Phone,
            login_identifier: phone.trim().to_string(),
            password: None,
            third_party_union_id: None,
            is_verified: req.is_verified,
            status: req.status,
        });
    }
    if let (Some(login_type), Some(login_identifier)) = (req.login_type, &req.login_identifier) {
        if !login_identifier.trim().is_empty() {
            logins.push(CreateLoginAccountReq {
                login_type,
                login_identifier: login_identifier.trim().to_string(),
                password: req.password.clone(),
                third_party_union_id: req.third_party_union_id.clone(),
                is_verified: req.is_verified,
                status: req.status,
            });
        }
    }
    logins
}

fn normalize_login_account(login: &mut CreateLoginAccountReq) {
    login.login_identifier = login.login_identifier.trim().to_string();
    if matches!(login.login_type, LoginType::Email | LoginType::Phone) {
        login.third_party_union_id = None;
    }
    if !login.login_type.needs_local_password() {
        login.password = None;
    }
}

fn login_account_for_save(req: &CreateLoginAccountReq) -> Result<LoginAccountForSave, AppError> {
    let mut login = req.clone();
    normalize_login_account(&mut login);
    let password_hash = login_password_hash(&login)?;
    Ok(LoginAccountForSave {
        login_type: login.login_type,
        login_identifier: login.login_identifier,
        password_hash,
        third_party_union_id: login.third_party_union_id,
        is_verified: login.is_verified.unwrap_or(0),
        status: login.status.unwrap_or(1),
    })
}

fn validate_profile_fields(
    real_name: &str,
    hospital_id: u64,
    dept_id: u64,
    identity_type: &str,
    status: Option<i32>,
) -> Result<(), AppError> {
    if real_name.trim().is_empty() {
        return Err(AppError::BadRequest("姓名不能为空".to_string()));
    }
    if hospital_id == 0 {
        return Err(AppError::BadRequest("医院不能为空".to_string()));
    }
    if dept_id == 0 {
        return Err(AppError::BadRequest("科室不能为空".to_string()));
    }
    if !matches!(identity_type, MEDICAL_WORKER | NON_MEDICAL_WORKER) {
        return Err(AppError::BadRequest("身份类型不正确".to_string()));
    }
    validate_status(status)
}

fn validate_login_identifier(login_identifier: &str) -> Result<(), AppError> {
    if login_identifier.trim().is_empty() {
        return Err(AppError::BadRequest("登录标识不能为空".to_string()));
    }
    Ok(())
}

fn validate_status(status: Option<i32>) -> Result<(), AppError> {
    if let Some(status) = status {
        if !matches!(status, 0 | 1) {
            return Err(AppError::BadRequest("状态只能是0或1".to_string()));
        }
    }
    Ok(())
}
