use crate::account::account_model::{
    AccountDetail, CreateAccountReq, CreateLoginAccountReq, LoginAccountForSave, LoginType,
    UpdateUserProfileReq,
};
use crate::account::account_repository;
use crate::common::cache;
use crate::common::constants::account::{
    IDENTITY_MEDICAL_WORKER, IDENTITY_NON_MEDICAL_WORKER, MAX_LOGIN_ACCOUNT_COUNT,
};
use crate::common::constants::auth::ROLE_USER;
use crate::common::jwt::Claims;
use crate::common::validation::validate_enabled_or_disabled;
use crate::error::AppError;
use crate::state::AppState;
use crate::wechat::wechat_model::WechatQrcodeRegisterReq;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand_core::OsRng;
use std::collections::HashSet;

/// 校验注册账号请求，包含用户资料和登录绑定规则。
pub fn validate_create_account_req(req: &CreateAccountReq) -> Result<(), AppError> {
    validate_profile_fields(
        &req.real_name,
        req.hospital_id,
        req.dept_id,
        req.identity_type.as_deref(),
        req.status,
    )?;
    let logins = account_login_reqs(req)?;
    validate_register_login_type_params(req, &logins)?;
    for login in logins {
        validate_create_login_account_req(&login)?;
    }
    Ok(())
}

/// 校验用户资料更新请求，不处理登录凭证字段。
pub fn validate_update_user_profile_req(req: &UpdateUserProfileReq) -> Result<(), AppError> {
    validate_profile_fields(
        &req.real_name,
        req.hospital_id,
        req.dept_id,
        req.identity_type.as_deref(),
        req.status,
    )
}

/// 校验单个登录绑定请求，例如邮箱密码、状态值和登录标识。
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
    validate_enabled_or_disabled(req.status, "状态只能是0或1")?;
    Ok(())
}

/// 校验注册时不同登录方式要求的附加参数。
fn validate_register_login_type_params(
    req: &CreateAccountReq,
    logins: &[CreateLoginAccountReq],
) -> Result<(), AppError> {
    for login in logins {
        match login.login_type {
            LoginType::Phone => {
                require_login_verification_code(req.verification_code.as_deref())?;
            }
            LoginType::Wechat | LoginType::Github => {
                require_third_party_union_id(login.third_party_union_id.as_deref())?;
            }
            LoginType::Email => {}
        }
    }
    Ok(())
}

/// 处理账号相关的业务转换。
pub fn account_login_reqs(req: &CreateAccountReq) -> Result<Vec<CreateLoginAccountReq>, AppError> {
    let mut logins = if req.login_accounts.is_empty() {
        legacy_login_reqs(req)
    } else {
        req.login_accounts.clone()
    };

    if logins.is_empty() {
        return Err(AppError::BadRequest("至少绑定一个登录账户".to_string()));
    }
    if logins.len() > MAX_LOGIN_ACCOUNT_COUNT {
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

/// 生成安全哈希值用于后续校验。
pub fn hash_password(password: &str) -> Result<String, AppError> {
    // Argon2 输出里包含算法参数和 salt，保存这一串即可支持后续 verify。
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Internal(err.to_string()))
}

/// 验证凭证或验证码是否有效。
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|err| AppError::Internal(err.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// 读取必填字段，缺失或空值时返回业务错误。
pub fn require_login_identifier(
    login_type: LoginType,
    login_identifier: Option<&str>,
) -> Result<String, AppError> {
    let identifier = login_identifier
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest("登录标识不能为空".to_string()))?;
    if matches!(login_type, LoginType::Wechat | LoginType::Github) {
        return Ok(identifier.to_string());
    }
    Ok(identifier.to_string())
}

/// 读取必填字段，缺失或空值时返回业务错误。
pub fn require_login_password(password: Option<&str>) -> Result<String, AppError> {
    password
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("密码不能为空".to_string()))
}

/// 读取必填字段，缺失或空值时返回业务错误。
pub fn require_login_verification_code(
    verification_code: Option<&str>,
) -> Result<String, AppError> {
    verification_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("验证码不能为空".to_string()))
}

/// 读取必填字段，缺失或空值时返回业务错误。
pub fn require_third_party_union_id(
    third_party_union_id: Option<&str>,
) -> Result<String, AppError> {
    third_party_union_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("thirdPartyUnionId不能为空".to_string()))
}

/// 校验需要验证的登录方式是否已经完成验证。
pub fn validate_verified_login_account(
    login_type: LoginType,
    is_verified: i32,
) -> Result<(), AppError> {
    match login_type {
        LoginType::Email if is_verified != 1 => {
            Err(AppError::Unauthorized("邮箱尚未验证".to_string()))
        }
        LoginType::Phone if is_verified != 1 => {
            Err(AppError::Unauthorized("手机号尚未验证".to_string()))
        }
        _ => Ok(()),
    }
}

/// 处理登录相关的业务转换。
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

/// 处理账号相关的业务转换。
pub fn account_token_subject(account: &AccountDetail) -> String {
    account
        .login_accounts
        .first()
        .map(|login| login.login_identifier.clone())
        .unwrap_or_else(|| account.profile.real_name.clone())
}

/// 读取必填字段，缺失或空值时返回业务错误。
pub fn require_claim_user_id(claims: &Claims) -> Result<u64, AppError> {
    claims
        .uid
        .ok_or_else(|| AppError::Unauthorized("Token missing user id".to_string()))
}

/// 创建业务数据，并返回创建后的记录。
pub async fn create_account(
    state: &mut AppState,
    req: CreateAccountReq,
) -> Result<crate::account::account_model::AccountDetail, AppError> {
    validate_create_account_req(&req)?;
    let login_accounts = account_login_reqs(&req)?
        .into_iter()
        .map(|login| login_account_for_save(&login))
        .collect::<Result<Vec<_>, _>>()?;
    // 用户资料和初始登录方式必须一起成功，避免产生无登录入口的用户资料。
    let id =
        account_repository::insert_account_with_logins(&state.db, &req, &login_accounts).await?;
    let account = account_repository::find_account_detail_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("account not found".to_string()))?;
    cache::cache_account(state, &account).await?;
    Ok(account)
}

/// 更新业务数据，并在目标不存在时返回 NotFound。
pub async fn update_profile(
    state: &mut AppState,
    id: u64,
    req: UpdateUserProfileReq,
) -> Result<bool, AppError> {
    validate_update_user_profile_req(&req)?;
    let ok = account_repository::update_user_profile(&state.db, id, &req).await?;
    cache::delete_account_cache(state, id).await?;
    Ok(ok)
}

/// 给当前用户新增一条登录绑定，并清理账号缓存。
pub async fn bind_account(
    state: &mut AppState,
    user_id: u64,
    req: CreateLoginAccountReq,
) -> Result<crate::account::account_model::UserLoginAccount, AppError> {
    validate_create_login_account_req(&req)?;
    // 只有邮箱会生成 password_hash；手机、微信、GitHub 的 password_hash 保持为空。
    let login = login_account_for_save(&req)?;
    let login_id = account_repository::insert_login_account(&state.db, user_id, &login).await?;
    cache::delete_account_cache(state, user_id).await?;
    account_repository::find_login_account_by_id(&state.db, user_id, login_id)
        .await?
        .ok_or_else(|| AppError::NotFound("login account not found".to_string()))
}

/// 逻辑删除用户资料，并清理对应账号缓存。
pub async fn delete_account(state: &mut AppState, id: u64) -> Result<bool, AppError> {
    let ok = account_repository::logical_delete_user(&state.db, id).await?;
    cache::delete_account_cache(state, id).await?;
    Ok(ok)
}

/// 删除当前用户的一条登录绑定，并清理账号缓存。
pub async fn unbind_account(
    state: &mut AppState,
    user_id: u64,
    login_id: u64,
) -> Result<bool, AppError> {
    let ok = account_repository::delete_login_account(&state.db, user_id, login_id).await?;
    cache::delete_account_cache(state, user_id).await?;
    Ok(ok)
}

/// 兼容旧请求结构，转换成当前统一模型。
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

/// 规范化请求字段，避免保存多余或不一致的数据。
fn normalize_login_account(login: &mut CreateLoginAccountReq) {
    login.login_identifier = login.login_identifier.trim().to_string();
    if matches!(login.login_type, LoginType::Email | LoginType::Phone) {
        login.third_party_union_id = None;
    }
    if !login.login_type.needs_local_password() {
        login.password = None;
    }
}

/// 处理登录相关的业务转换。
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

/// 校验用户资料公共字段，例如姓名、医院、科室、身份和状态。
fn validate_profile_fields(
    real_name: &str,
    hospital_id: Option<u64>,
    dept_id: Option<u64>,
    identity_type: Option<&str>,
    status: Option<i32>,
) -> Result<(), AppError> {
    if real_name.trim().is_empty() {
        return Err(AppError::BadRequest("姓名不能为空".to_string()));
    }
    if hospital_id.is_some_and(|value| value == 0) {
        return Err(AppError::BadRequest("医院不正确".to_string()));
    }
    if dept_id.is_some_and(|value| value == 0) {
        return Err(AppError::BadRequest("科室不正确".to_string()));
    }

    if let Some(identity_type) = identity_type.map(str::trim) {
        if !matches!(
            identity_type,
            IDENTITY_MEDICAL_WORKER | IDENTITY_NON_MEDICAL_WORKER
        ) {
            return Err(AppError::BadRequest("身份类型不正确".to_string()));
        }
    }
    validate_enabled_or_disabled(status, "状态只能是0或1")
}

/// 校验登录标识不能为空。
fn validate_login_identifier(login_identifier: &str) -> Result<(), AppError> {
    if login_identifier.trim().is_empty() {
        return Err(AppError::BadRequest("登录标识不能为空".to_string()));
    }
    Ok(())
}

/// 根据微信 openId 登录。
///
/// 如果 openId 已经绑定过用户：
/// 1. 更新最后登录时间。
/// 2. 直接签发 JWT。
///
/// 如果 openId 第一次出现：
/// 1. 创建一个资料未补全的 user_info。
/// 2. 创建 WECHAT 登录绑定。
/// 3. 签发 JWT。
pub async fn login_or_create_by_wechat(
    state: &mut AppState,
    open_id: &str,
    union_id: Option<&str>,
) -> Result<String, AppError> {
    tracing::info!(
        open_id = %open_id,
        union_id = ?union_id,
        "login_or_create_by_wechat started"
    );
    let existing_login = account_repository::find_wechat_login_for_auth(&state.db, open_id).await?;
    let user_id = if let Some(login) = existing_login {
        tracing::info!(
            user_id = login.user_id,
            "wechat account found, touch last_login_at"
        );
        account_repository::touch_last_login(&state.db, login.user_id, open_id).await?;
        login.user_id
    } else {
        tracing::info!("wechat account not found, create new user");
        account_repository::insert_wechat_user(&state.db, open_id, union_id).await?
    };
    let account = account_repository::find_account_detail_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("登录账户不可用".to_string()))?;
    let uid = account
        .profile
        .id
        .ok_or_else(|| AppError::Internal("login account has no user id".to_string()))?;
    let token = state.jwt.generate_token(
        &account_token_subject(&account),
        vec![ROLE_USER.to_string()],
        Some(uid),
    )?;
    cache::cache_token(state, &account, &token).await?;
    tracing::info!(
        user_id = user_id,
        token_len = token.len(),
        "login_or_create_by_wechat finished"
    );
    Ok(token)
}

/// 校验并规范化微信扫码注册资料。
///
/// 姓名、昵称和身份类型始终必填；医疗从业者还必须选择医院和科室。
/// mobile 和 headerId 只作为用户资料保存，不创建新的登录方式。
pub fn validate_wechat_qrcode_register_req(
    req: &mut WechatQrcodeRegisterReq,
) -> Result<(), AppError> {
    req.real_name = req.real_name.trim().to_string();
    req.nickname = req.nickname.trim().to_string();
    req.identity_type = req.identity_type.trim().to_ascii_uppercase();

    req.mobile = req
        .mobile
        .take()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if req.real_name.is_empty() {
        return Err(AppError::BadRequest("姓名不能为空".to_string()));
    }
    if req.real_name.chars().count() > 128 {
        return Err(AppError::BadRequest("姓名不能超过128个字符".to_string()));
    }

    if req.nickname.is_empty() {
        return Err(AppError::BadRequest("昵称不能为空".to_string()));
    }
    if req.nickname.chars().count() > 128 {
        return Err(AppError::BadRequest("昵称不能超过128个字符".to_string()));
    }

    if let Some(mobile) = &req.mobile {
        if mobile.chars().count() > 30 {
            return Err(AppError::BadRequest("联系电话不能超过30个字符".to_string()));
        }
    }

    if req.header_id.is_some_and(|value| value == 0) {
        return Err(AppError::BadRequest("头像文件ID不正确".to_string()));
    }
    for (file_id, label) in [
        (req.doctor_cert_file_id, "执业资格证文件ID不正确"),
        (req.id_card_front_file_id, "身份证人像面文件ID不正确"),
        (req.id_card_back_file_id, "身份证国徽面文件ID不正确"),
    ] {
        if file_id == Some(0) {
            return Err(AppError::BadRequest(label.to_string()));
        }
    }

    // 身份类型决定医院和科室是否为业务必填项。
    match req.identity_type.as_str() {
        IDENTITY_MEDICAL_WORKER => {
            if req.hospital_id.is_none_or(|value| value == 0) {
                return Err(AppError::BadRequest("医疗从业者必须选择医院".to_string()));
            }
            if req.dept_id.is_none_or(|value| value == 0) {
                return Err(AppError::BadRequest("医疗从业者必须选择科室".to_string()));
            }
        }
        IDENTITY_NON_MEDICAL_WORKER => {
            if req.hospital_id.is_some_and(|value| value == 0) {
                return Err(AppError::BadRequest("医院不正确".to_string()));
            }
            if req.dept_id.is_some_and(|value| value == 0) {
                return Err(AppError::BadRequest("科室不正确".to_string()));
            }
        }
        _ => {
            return Err(AppError::BadRequest(
                "身份类型只支持MEDICAL_WORKER或NON_MEDICAL_WORKER".to_string(),
            ));
        }
    }

    Ok(())
}

/// 给已确认的账号签发并缓存 JWT。
///
/// 不在日志中打印 JWT。
pub async fn issue_account_token(
    state: &mut AppState,
    account: &AccountDetail,
) -> Result<String, AppError> {
    let user_id = account
        .profile
        .id
        .ok_or_else(|| AppError::Internal("account has no user id".to_string()))?;

    let token = state.jwt.generate_token(
        &account_token_subject(account),
        vec![ROLE_USER.to_string()],
        Some(user_id),
    )?;

    cache::cache_token(state, account, &token).await?;

    tracing::info!(user_id, token_len = token.len(), "account token issued");

    Ok(token)
}

/// 微信扫码时，如果账号已经存在则直接登录；不存在时返回 None。
///
/// 这个方法绝不会自动创建 user_info。
pub async fn login_by_wechat_if_exists(
    state: &mut AppState,
    open_id: &str,
) -> Result<Option<(u64, String)>, AppError> {
    tracing::info!(
        open_id = %open_id,
        "login_by_wechat_if_exists started"
    );
    let Some(login) = account_repository::find_wechat_login_for_auth(&state.db, open_id).await?
    else {
        tracing::info!("wechat account not found");
        return Ok(None);
    };

    account_repository::touch_last_login(&state.db, login.user_id, &login.login_identifier).await?;

    let account = account_repository::find_account_detail_by_id(&state.db, login.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("微信登录账号不可用".to_string()))?;

    let token = issue_account_token(state, &account).await?;

    tracing::info!(
        user_id = login.user_id,
        "existing wechat account login succeeded"
    );

    Ok(Some((login.user_id, token)))
}

/// 使用服务端保存的微信注册上下文创建正式账号。
///
/// 资料校验通过后，Repository 会在同一事务内写入用户资料和微信登录绑定。
pub async fn create_wechat_account(
    state: &mut AppState,
    mut req: WechatQrcodeRegisterReq,
    open_id: &str,
    union_id: Option<&str>,
) -> Result<AccountDetail, AppError> {
    tracing::info!(
        open_id = %open_id,
        union_id = ?union_id,
        real_name = %req.real_name,
        nickname = %req.nickname,
        identity_type = %req.identity_type,
        hospital_id = ?req.hospital_id,
        dept_id = ?req.dept_id,
        mobile = ?req.mobile,
        header_id = ?req.header_id,
        doctor_cert_file_id = ?req.doctor_cert_file_id,
        id_card_front_file_id = ?req.id_card_front_file_id,
        id_card_back_file_id = ?req.id_card_back_file_id,
        "create_wechat_account started"
    );
    validate_wechat_qrcode_register_req(&mut req)?;

    let user_id =
        account_repository::insert_wechat_account(&state.db, &req, open_id, union_id).await?;

    let account = account_repository::find_account_detail_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::Internal("微信账号创建成功但无法读取用户资料".to_string()))?;

    cache::cache_account(state, &account).await?;

    tracing::info!(user_id, "wechat account created after profile completion");

    Ok(account)
}
