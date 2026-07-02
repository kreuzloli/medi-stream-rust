use crate::common::constants::{
    http::{CONTENT_TYPE_JSON_UTF8, HEADER_CONTENT_TYPE, HEADER_HOST},
    tencent_cloud::{TENCENT_CLOUD_ALGORITHM, TENCENT_LIVE_HOST, TENCENT_LIVE_SERVICE},
};
use crate::error::AppError;
use chrono::{TimeZone, Utc};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct LiveCredential {
    pub secret_id: String,
    pub secret_key: String,
}

/// 按腾讯云 TC3-HMAC-SHA256 规则构造 Authorization 头。
pub fn build_live_authorization<T>(
    credential: &LiveCredential,
    timestamp: i64,
    req: &T,
) -> Result<String, AppError>
where
    T: Serialize,
{
    let payload = serde_json::to_string(req)?;
    let payload_hash = sha256_hex(payload.as_bytes());
    let canonical_request = format!(
        "POST\n/\n\n{}:{}\n{}:{}\n\ncontent-type;host\n{}",
        HEADER_CONTENT_TYPE.to_ascii_lowercase(),
        CONTENT_TYPE_JSON_UTF8,
        HEADER_HOST.to_ascii_lowercase(),
        TENCENT_LIVE_HOST,
        payload_hash
    );

    let request_hash = sha256_hex(canonical_request.as_bytes());
    let date = Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or_else(|| AppError::BadRequest("invalid Tencent live timestamp".to_string()))?
        .format("%Y-%m-%d")
        .to_string();
    let credential_scope = format!("{}/{}/tc3_request", date, TENCENT_LIVE_SERVICE);
    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        TENCENT_CLOUD_ALGORITHM, timestamp, credential_scope, request_hash
    );

    let secret_date = hmac_sha256(
        format!("TC3{}", credential.secret_key).as_bytes(),
        date.as_bytes(),
    )?;
    let secret_service = hmac_sha256(&secret_date, TENCENT_LIVE_SERVICE.as_bytes())?;
    let secret_signing = hmac_sha256(&secret_service, b"tc3_request")?;
    let signature = hex_lower(&hmac_sha256(&secret_signing, string_to_sign.as_bytes())?);

    Ok(format!(
        "{} Credential={}/{}, SignedHeaders=content-type;host, Signature={}",
        TENCENT_CLOUD_ALGORITHM, credential.secret_id, credential_scope, signature
    ))
}

/// 计算 SHA-256 小写十六进制摘要。
fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    hex_lower(&digest)
}

/// 使用 HMAC-SHA256 对数据签名。
fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|err| AppError::Internal(format!("create Tencent live signer failed: {err}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// 把字节数组转换成小写十六进制字符串。
fn hex_lower(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
