use crate::account::model::{
    AccountDetail, AccountPageQuery, AuthLoginAccount, CreateAccountReq, LoginAccountForSave,
    LoginType, UpdateUserProfileReq, UserLoginAccount, UserProfile,
};
use crate::common::Page;
use crate::error::AppError;
use sqlx::{MySql, MySqlPool, QueryBuilder, Row};
use uuid::Uuid;

pub async fn insert_account_with_logins(
    db: &MySqlPool,
    req: &CreateAccountReq,
    login_accounts: &[LoginAccountForSave],
) -> Result<u64, AppError> {
    // 注册要同时写 user_info 和 user_login_account；任一失败都回滚，避免半成品账号。
    let mut tx = db.begin().await?;

    for login in login_accounts {
        let exists = sqlx::query(
            r#"
            SELECT id
            FROM user_login_account
            WHERE login_type = ? AND login_identifier = ?
            LIMIT 1
            "#,
        )
        .bind(login.login_type.as_str())
        .bind(&login.login_identifier)
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if exists {
            return Err(AppError::BadRequest("该账户已经存在".to_string()));
        }
    }

    let user_code = Uuid::new_v4().simple().to_string();
    let user_result = sqlx::query(
        r#"
        INSERT INTO user_info (
            user_code, real_name, nickname, hospital_id, dept_id, identity_type,
            doctor_cert_no, id_card_no, status, version, is_deleted
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0)
        "#,
    )
    .bind(user_code)
    .bind(&req.real_name)
    .bind(&req.nickname)
    .bind(req.hospital_id)
    .bind(req.dept_id)
    .bind(&req.identity_type)
    .bind(&req.doctor_cert_no)
    .bind(&req.id_card_no)
    .bind(req.status.unwrap_or(1))
    .execute(&mut *tx)
    .await?;
    let user_id = user_result.last_insert_id();

    for login in login_accounts {
        sqlx::query(
            r#"
            INSERT INTO user_login_account (
                user_id, login_type, login_identifier, password_hash, third_party_union_id,
                is_verified, status, is_deleted
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, 0)
            "#,
        )
        .bind(user_id)
        .bind(login.login_type.as_str())
        .bind(&login.login_identifier)
        .bind(&login.password_hash)
        .bind(&login.third_party_union_id)
        .bind(login.is_verified)
        .bind(login.status)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(user_id)
}

pub async fn insert_login_account(
    db: &MySqlPool,
    user_id: u64,
    login: &LoginAccountForSave,
) -> Result<u64, AppError> {
    let exists = sqlx::query(
        r#"
        SELECT id
        FROM user_login_account
        WHERE login_type = ? AND login_identifier = ?
        LIMIT 1
        "#,
    )
    .bind(login.login_type.as_str())
    .bind(&login.login_identifier)
    .fetch_optional(db)
    .await?;
    if exists.is_some() {
        return Err(AppError::BadRequest("该账户已经存在".to_string()));
    }

    let result = sqlx::query(
        r#"
        INSERT INTO user_login_account (
            user_id, login_type, login_identifier, password_hash, third_party_union_id,
            is_verified, status, is_deleted
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, 0)
        "#,
    )
    .bind(user_id)
    .bind(login.login_type.as_str())
    .bind(&login.login_identifier)
    .bind(&login.password_hash)
    .bind(&login.third_party_union_id)
    .bind(login.is_verified)
    .bind(login.status)
    .execute(db)
    .await?;

    Ok(result.last_insert_id())
}

pub async fn find_account_detail_by_id(
    db: &MySqlPool,
    id: u64,
) -> Result<Option<AccountDetail>, AppError> {
    let Some(profile) = find_user_profile_by_id(db, id).await? else {
        return Ok(None);
    };
    let login_accounts = find_login_accounts_by_user_id(db, id).await?;
    Ok(Some(AccountDetail {
        profile,
        login_accounts,
    }))
}

pub async fn find_user_profile_by_id(
    db: &MySqlPool,
    id: u64,
) -> Result<Option<UserProfile>, AppError> {
    Ok(sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT
            id, user_code, real_name, nickname, hospital_id, dept_id, identity_type,
            doctor_cert_no, id_card_no, status, version, is_deleted, created_at, updated_at
        FROM user_info
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

pub async fn find_login_account_by_id(
    db: &MySqlPool,
    user_id: u64,
    login_id: u64,
) -> Result<Option<UserLoginAccount>, AppError> {
    Ok(sqlx::query_as::<_, UserLoginAccount>(
        r#"
        SELECT
            id, user_id, login_type, login_identifier, third_party_union_id,
            is_verified, last_login_at, status, is_deleted, created_at, updated_at
        FROM user_login_account
        WHERE id = ? AND user_id = ? AND is_deleted = 0
        "#,
    )
    .bind(login_id)
    .bind(user_id)
    .fetch_optional(db)
    .await?)
}

pub async fn find_login_accounts_by_user_id(
    db: &MySqlPool,
    user_id: u64,
) -> Result<Vec<UserLoginAccount>, AppError> {
    Ok(sqlx::query_as::<_, UserLoginAccount>(
        r#"
        SELECT
            id, user_id, login_type, login_identifier, third_party_union_id,
            is_verified, last_login_at, status, is_deleted, created_at, updated_at
        FROM user_login_account
        WHERE user_id = ? AND is_deleted = 0
        ORDER BY id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

pub async fn update_user_profile(
    db: &MySqlPool,
    id: u64,
    req: &UpdateUserProfileReq,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE user_info
        SET
            user_code = ?, real_name = ?, nickname = ?, hospital_id = ?, dept_id = ?,
            identity_type = ?, doctor_cert_no = ?, id_card_no = ?,
            status = COALESCE(?, status), version = COALESCE(?, version)
        WHERE id = ? AND is_deleted = 0
        "#,
    )
    .bind(&req.user_code)
    .bind(&req.real_name)
    .bind(&req.nickname)
    .bind(req.hospital_id)
    .bind(req.dept_id)
    .bind(&req.identity_type)
    .bind(&req.doctor_cert_no)
    .bind(&req.id_card_no)
    .bind(req.status)
    .bind(req.version)
    .bind(id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn logical_delete_user(db: &MySqlPool, id: u64) -> Result<bool, AppError> {
    // 用户资料仍保留逻辑删除；登录绑定按“只保存/删除”的规则做物理删除。
    let mut tx = db.begin().await?;
    let result = sqlx::query(
        "UPDATE user_info SET status = 0, is_deleted = 1 WHERE id = ? AND is_deleted = 0",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() > 0 {
        sqlx::query("DELETE FROM user_login_account WHERE user_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_login_account(
    db: &MySqlPool,
    user_id: u64,
    login_id: u64,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        DELETE FROM user_login_account
        WHERE id = ? AND user_id = ?
        "#,
    )
    .bind(login_id)
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn page_user_profiles(
    db: &MySqlPool,
    query: AccountPageQuery,
) -> Result<Page<UserProfile>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(10).clamp(1, 200);
    let offset = (page - 1) * size;

    let mut data_query = QueryBuilder::<MySql>::new(
        "SELECT id, user_code, real_name, nickname, hospital_id, dept_id, identity_type, \
         doctor_cert_no, id_card_no, status, version, is_deleted, created_at, updated_at \
         FROM user_info WHERE is_deleted = 0",
    );
    push_profile_filters(&mut data_query, &query);
    data_query.push(" ORDER BY id DESC LIMIT ");
    data_query.push_bind(size);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let records = data_query
        .build_query_as::<UserProfile>()
        .fetch_all(db)
        .await?;

    let mut count_query =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) AS total FROM user_info WHERE is_deleted = 0");
    push_profile_filters(&mut count_query, &query);
    let total: i64 = count_query.build().fetch_one(db).await?.try_get("total")?;
    let total = total.max(0) as u64;

    Ok(Page {
        records,
        total,
        size,
        current: page,
        pages: if total == 0 {
            0
        } else {
            (total + size - 1) / size
        },
    })
}

pub async fn find_login_for_auth(
    db: &MySqlPool,
    login_type: LoginType,
    login_identifier: &str,
) -> Result<Option<AuthLoginAccount>, AppError> {
    // 登录必须同时满足绑定有效和用户资料有效，防止停用用户通过旧绑定进入系统。
    Ok(sqlx::query_as::<_, AuthLoginAccount>(
        r#"
        SELECT
            login.user_id, login.login_identifier, login.password_hash
        FROM user_login_account login
        INNER JOIN user_info user ON user.id = login.user_id
        WHERE login.login_type = ?
          AND login.login_identifier = ?
          AND login.status = 1
          AND login.is_deleted = 0
          AND user.status = 1
          AND user.is_deleted = 0
        "#,
    )
    .bind(login_type.as_str())
    .bind(login_identifier)
    .fetch_optional(db)
    .await?)
}

fn push_profile_filters(query_builder: &mut QueryBuilder<MySql>, query: &AccountPageQuery) {
    if let Some(user_code) = query
        .user_code
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND user_code LIKE ");
        query_builder.push_bind(format!("%{}%", user_code.trim()));
    }
    if let Some(real_name) = query
        .real_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND real_name LIKE ");
        query_builder.push_bind(format!("%{}%", real_name.trim()));
    }
    if let Some(hospital_id) = query.hospital_id {
        query_builder.push(" AND hospital_id = ");
        query_builder.push_bind(hospital_id);
    }
    if let Some(dept_id) = query.dept_id {
        query_builder.push(" AND dept_id = ");
        query_builder.push_bind(dept_id);
    }
    if let Some(identity_type) = query
        .identity_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        query_builder.push(" AND identity_type = ");
        query_builder.push_bind(identity_type.trim().to_string());
    }
    if let Some(status) = query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }
}
