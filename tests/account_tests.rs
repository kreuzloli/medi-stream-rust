use medi_stream_rust::account::model::{
    AccountDetail, CreateAccountReq, CreateLoginAccountReq, LoginType, UserLoginAccount,
    UserProfile,
};
use medi_stream_rust::account::service::{
    account_login_reqs, account_token_subject, hash_password, login_password_hash,
    require_claim_user_id, validate_create_account_req, validate_create_login_account_req,
    verify_password,
};
use medi_stream_rust::auth::jwt::Claims;

#[test]
fn create_account_requires_profile_and_password_fields() {
    let mut req = valid_create_req();
    req.real_name = "  ".to_string();

    let err = validate_create_account_req(&req).expect_err("blank real name must be rejected");

    assert!(err.to_string().contains("姓名不能为空"));
}

#[test]
fn create_account_accepts_required_medical_profile_fields() {
    let req = valid_create_req();

    validate_create_account_req(&req).expect("valid account request should pass validation");
}

#[test]
fn create_account_can_bind_all_supported_login_types() {
    let req = CreateAccountReq {
        login_accounts: vec![
            login_req(
                LoginType::Email,
                "doctor@example.com",
                Some("secret-123456"),
            ),
            login_req(LoginType::Phone, "13800138000", None),
            login_req(LoginType::Wechat, "wechat-openid-001", None),
            login_req(LoginType::Github, "github-user-001", None),
        ],
        ..valid_create_req()
    };

    let logins = account_login_reqs(&req).expect("login accounts should be normalized");

    assert_eq!(logins.len(), 4);
    assert!(logins
        .iter()
        .any(|login| login.login_type == LoginType::Email));
    assert!(logins
        .iter()
        .any(|login| login.login_type == LoginType::Phone));
    assert!(logins
        .iter()
        .any(|login| login.login_type == LoginType::Wechat));
    assert!(logins
        .iter()
        .any(|login| login.login_type == LoginType::Github));
}

#[test]
fn only_email_login_requires_password() {
    let email_without_password = login_req(LoginType::Email, "doctor@example.com", None);
    let phone_without_password = login_req(LoginType::Phone, "13800138000", None);
    let wechat_without_password = login_req(LoginType::Wechat, "wechat-openid-001", None);
    let github_without_password = login_req(LoginType::Github, "github-user-001", None);

    let err = validate_create_login_account_req(&email_without_password)
        .expect_err("email login must require password");

    assert!(err.to_string().contains("邮箱登录必须填写密码"));
    validate_create_login_account_req(&phone_without_password)
        .expect("phone does not need password");
    validate_create_login_account_req(&wechat_without_password)
        .expect("wechat does not need password");
    validate_create_login_account_req(&github_without_password)
        .expect("github does not need password");
}

#[test]
fn password_hash_is_created_for_email_only() {
    let email = login_req(
        LoginType::Email,
        "doctor@example.com",
        Some("secret-123456"),
    );
    let phone = login_req(LoginType::Phone, "13800138000", None);

    let email_hash = login_password_hash(&email).expect("email hash should be created");
    let phone_hash = login_password_hash(&phone).expect("phone should not need password hash");

    assert!(email_hash.is_some());
    assert!(phone_hash.is_none());
}

#[test]
fn register_token_subject_uses_first_login_identifier() {
    let account = account_detail(Some(100), vec![login_account("doctor@example.com")]);

    let subject = account_token_subject(&account);

    assert_eq!(subject, "doctor@example.com");
}

#[test]
fn get_account_requires_uid_from_jwt_claims() {
    let claims = Claims {
        iss: "medi-stream".to_string(),
        sub: "doctor@example.com".to_string(),
        iat: 1,
        exp: 2,
        roles: vec!["USER".to_string()],
        uid: Some(100),
    };

    let uid = require_claim_user_id(&claims).expect("uid should be available");

    assert_eq!(uid, 100);
}

#[test]
fn login_type_serializes_as_database_value() {
    assert_eq!(LoginType::Email.as_str(), "EMAIL");
    assert_eq!(LoginType::Phone.as_str(), "PHONE");
    assert_eq!(LoginType::Wechat.as_str(), "WECHAT");
    assert_eq!(LoginType::Github.as_str(), "GITHUB");
}

#[test]
fn password_hash_does_not_store_plain_text_and_can_be_verified() {
    let password = "secret-123456";

    let hash = hash_password(password).expect("password hash should be created");

    assert_ne!(hash, password);
    assert!(verify_password(password, &hash).expect("password verification should run"));
    assert!(!verify_password("wrong-password", &hash).expect("password verification should run"));
}

fn valid_create_req() -> CreateAccountReq {
    CreateAccountReq {
        user_code: Some("U001".to_string()),
        real_name: "张三".to_string(),
        nickname: Some("医生张".to_string()),
        hospital_id: 1,
        dept_id: 2,
        identity_type: "MEDICAL_WORKER".to_string(),
        doctor_cert_no: Some("CERT001".to_string()),
        id_card_no: Some("110101199001011234".to_string()),
        login_type: Some(LoginType::Email),
        login_identifier: Some("doctor@example.com".to_string()),
        password: Some("secret-123456".to_string()),
        third_party_union_id: None,
        is_verified: Some(1),
        status: Some(1),
        email: None,
        phone: None,
        verification_code: None,
        login_accounts: Vec::new(),
    }
}

fn login_req(
    login_type: LoginType,
    login_identifier: &str,
    password: Option<&str>,
) -> CreateLoginAccountReq {
    CreateLoginAccountReq {
        login_type,
        login_identifier: login_identifier.to_string(),
        password: password.map(str::to_string),
        third_party_union_id: None,
        is_verified: Some(1),
        status: Some(1),
    }
}

fn account_detail(id: Option<u64>, login_accounts: Vec<UserLoginAccount>) -> AccountDetail {
    AccountDetail {
        profile: UserProfile {
            id,
            user_code: Some("U001".to_string()),
            real_name: "张三".to_string(),
            nickname: None,
            hospital_id: 1,
            dept_id: 2,
            identity_type: "MEDICAL_WORKER".to_string(),
            doctor_cert_no: None,
            id_card_no: None,
            status: 1,
            version: 0,
            is_deleted: 0,
            created_at: None,
            updated_at: None,
        },
        login_accounts,
    }
}

fn login_account(login_identifier: &str) -> UserLoginAccount {
    UserLoginAccount {
        id: 1,
        user_id: 100,
        login_type: "EMAIL".to_string(),
        login_identifier: login_identifier.to_string(),
        third_party_union_id: None,
        is_verified: 1,
        last_login_at: None,
        status: 1,
        is_deleted: 0,
        created_at: None,
        updated_at: None,
    }
}
