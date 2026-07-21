use crate::config::FileStorageConfig;
use crate::error::AppError;
use axum::extract::multipart::Field;
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// 已经成功写入本地磁盘的文件。
#[derive(Debug)]
pub struct StoredFile {
    /// 用户上传时的原始文件名。
    pub original_name: String,

    /// 浏览器可以访问的相对路径。
    ///
    /// 例如：
    ///
    /// /uploads/550e8400-e29b-41d4-a716-446655440000.jpg
    pub public_url: String,

    /// 文件在服务器上的完整物理路径。
    ///
    /// 例如：
    ///
    /// /var/lib/medi-stream/uploads/550e8400-e29b-41d4-a716-446655440000.jpg
    pub absolute_path: PathBuf,

    /// Multipart 中传入的 MIME 类型。
    ///
    /// 例如：
    ///
    /// image/jpeg
    pub mime_type: String,

    /// 文件大小，单位为字节。
    pub file_size: u64,

    /// 文件内容的 SHA-256。
    pub sha256: String,
}

/// 把 Multipart 中的一份文件写入本地磁盘。
///
/// 文件直接保存到 FILE_STORAGE_ROOT 根目录。
///
/// 保存流程：
///
/// 1. 读取原始文件名和 MIME 类型；
/// 2. 生成 UUID 文件名；
/// 3. 先写入 `.tmp` 临时目录；
/// 4. 边读取边计算文件大小和 SHA-256；
/// 5. 超过配置大小时中止并删除临时文件；
/// 6. 写入成功后移动到正式目录。
pub async fn save_uploaded_file(
    mut field: Field<'_>,
    config: &FileStorageConfig,
) -> Result<StoredFile, AppError> {
    // 获取用户上传时的原始文件名。
    let original_name = field
        .file_name()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("上传文件缺少文件名".to_string()))?;

    // 获取客户端传入的 MIME 类型。
    //
    // 客户端没有传时，使用通用二进制类型。
    let mime_type = field
        .content_type()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream")
        .to_string();

    // 正式文件直接放在上传根目录。
    //
    // 例如：
    //
    // /var/lib/medi-stream/uploads
    let target_directory = &config.root_dir;

    // 上传中的临时文件单独放在 .tmp 目录。
    //
    // 这样即使上传中断，也不会在正式目录里留下半截文件。
    let temp_directory = config.root_dir.join(".tmp");

    // 确保存储目录存在。
    fs::create_dir_all(target_directory).await?;
    fs::create_dir_all(&temp_directory).await?;

    // 从原始文件名中提取安全扩展名。
    let extension = safe_extension(&original_name);

    // 使用 UUID 作为正式存储文件名，避免：
    //
    // 1. 文件名重复；
    // 2. 中文或特殊字符路径问题；
    // 3. 用户文件名造成路径穿越。
    let stored_file_name = match extension {
        Some(extension) => {
            format!("{}.{}", Uuid::new_v4(), extension)
        }
        None => Uuid::new_v4().to_string(),
    };

    // 正式文件完整路径。
    let target_path = target_directory.join(&stored_file_name);

    // 临时文件完整路径。
    let temp_path = temp_directory.join(format!("{}.uploading", Uuid::new_v4()));

    // 创建临时文件。
    let mut temp_file = File::create(&temp_path).await?;

    // 用于逐块计算 SHA-256。
    let mut sha256 = Sha256::new();

    // 已经接收的文件大小。
    let mut file_size: u64 = 0;

    // 单独保存写入结果，失败时可以统一删除临时文件。
    let write_result: Result<(), AppError> = async {
        // Multipart 文件不会一次性全部读入内存，
        // 而是按 chunk 分块读取。
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|error| AppError::BadRequest(format!("读取上传文件失败：{error}")))?
        {
            let chunk_size = u64::try_from(chunk.len())
                .map_err(|_| AppError::Internal("文件大小转换失败".to_string()))?;

            // 使用 checked_add 防止极端情况下整数溢出。
            file_size = file_size
                .checked_add(chunk_size)
                .ok_or_else(|| AppError::BadRequest("上传文件大小超出支持范围".to_string()))?;

            // 文件大小限制在 Rust API 中判断。
            //
            // Nginx 可以不限制，但这里会在接收过程中及时中止。
            if file_size > config.max_size_bytes {
                return Err(AppError::BadRequest(format!(
                    "上传文件不能超过 {}",
                    format_file_size(config.max_size_bytes)
                )));
            }

            // 当前数据块加入 SHA-256 计算。
            sha256.update(&chunk);

            // 当前数据块写入临时文件。
            temp_file.write_all(&chunk).await?;
        }

        // 把用户态缓冲区中的内容写入操作系统。
        temp_file.flush().await?;

        // 尽量确保内容真正同步到磁盘。
        temp_file.sync_all().await?;

        Ok(())
    }
    .await;

    // 上传或写入失败时，删除临时文件。
    if let Err(error) = write_result {
        remove_file_if_exists(&temp_path).await;
        return Err(error);
    }

    // 不允许上传空文件。
    if file_size == 0 {
        remove_file_if_exists(&temp_path).await;

        return Err(AppError::BadRequest("上传文件不能为空".to_string()));
    }

    // 把临时文件移动到正式目录。
    //
    // 在同一个文件系统内，rename 通常是原子操作。
    if let Err(error) = fs::rename(&temp_path, &target_path).await {
        remove_file_if_exists(&temp_path).await;
        return Err(AppError::Io(error));
    }

    // 生成数据库中保存的公开访问地址。
    //
    // 例如：
    //
    // /uploads/550e8400-e29b-41d4-a716-446655440000.jpg
    let public_url = format!(
        "{}/{}",
        config.public_prefix.trim_end_matches('/'),
        stored_file_name
    );

    Ok(StoredFile {
        original_name,
        public_url,
        absolute_path: target_path,
        mime_type,
        file_size,
        sha256: format!("{:x}", sha256.finalize()),
    })
}

/// 根据数据库中的 file_url 删除本地文件。
///
/// 只允许删除 FILE_STORAGE_ROOT 目录中的安全相对路径。
///
/// 如果 file_url 不是当前本地上传路径，例如远程 URL，
/// 则不会操作本地磁盘。
pub async fn delete_public_file(
    file_url: &str,
    config: &FileStorageConfig,
) -> Result<bool, AppError> {
    let Some(absolute_path) = public_file_path(file_url, config)? else {
        return Ok(false);
    };

    match fs::remove_file(&absolute_path).await {
        Ok(()) => Ok(true),

        // 文件已经不存在时，不当作业务失败。
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                path = %absolute_path.display(),
                "local file already missing"
            );

            Ok(false)
        }

        Err(error) => Err(AppError::Io(error)),
    }
}

/// 读取当前服务管理的本地文件内容。
///
/// 数据库中的公开 URL 会先经过前缀和相对路径校验，再映射到共享存储根目录，
/// 防止通过伪造 file_url 读取根目录之外的文件。
pub async fn read_public_file(
    file_url: &str,
    config: &FileStorageConfig,
) -> Result<Vec<u8>, AppError> {
    let absolute_path = public_file_path(file_url, config)?
        .ok_or_else(|| AppError::NotFound("文件不属于本地存储".to_string()))?;

    match fs::read(&absolute_path).await {
        Ok(content) => Ok(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(path = %absolute_path.display(), "local file content missing");
            Err(AppError::NotFound("文件不存在".to_string()))
        }
        Err(error) => Err(AppError::Io(error)),
    }
}

/// 将数据库公开 URL 安全映射为共享存储目录中的物理路径。
fn public_file_path(
    file_url: &str,
    config: &FileStorageConfig,
) -> Result<Option<PathBuf>, AppError> {
    let prefix = format!("{}/", config.public_prefix.trim_end_matches('/'));
    let Some(relative_path) = file_url.strip_prefix(&prefix) else {
        return Ok(None);
    };
    let relative_path = Path::new(relative_path);
    if !is_safe_relative_path(relative_path) {
        return Err(AppError::Internal(
            "文件记录包含不安全的本地路径".to_string(),
        ));
    }
    Ok(Some(config.root_dir.join(relative_path)))
}

/// 数据库保存失败时，删除已经写入磁盘的文件。
pub async fn rollback_stored_file(path: &Path) {
    remove_file_if_exists(path).await;
}

/// 从原始文件名中提取安全扩展名。
///
/// 仅允许：
///
/// - ASCII 英文字母；
/// - 数字；
/// - 最大长度 20。
///
/// 示例：
///
/// cover.JPG -> jpg
///
/// report.final.pdf -> pdf
///
/// 非法扩展名会被丢弃，最终文件将不带扩展名。
fn safe_extension(file_name: &str) -> Option<String> {
    let extension = Path::new(file_name)
        .extension()?
        .to_str()?
        .trim()
        .to_ascii_lowercase();

    if extension.is_empty() || extension.len() > 20 {
        return None;
    }

    if !extension
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }

    Some(extension)
}

/// 判断路径是否为安全相对路径。
///
/// 由于现在没有日期目录，正常情况下这里通常只有一个文件名，
/// 但仍保留完整路径检查，防止以后目录结构变化时埋雷。
fn is_safe_relative_path(path: &Path) -> bool {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return false;
    }

    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

/// 将字节数格式化为适合错误提示的文本。
///
/// 示例：
///
/// 104857600 -> 100MB
///
/// 1073741824 -> 1GB
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB && bytes % GB == 0 {
        format!("{}GB", bytes / GB)
    } else if bytes >= MB && bytes % MB == 0 {
        format!("{}MB", bytes / MB)
    } else if bytes >= KB && bytes % KB == 0 {
        format!("{}KB", bytes / KB)
    } else {
        format!("{bytes}字节")
    }
}

/// 删除文件。
///
/// 文件不存在时忽略，其他错误只记录日志。
async fn remove_file_if_exists(path: &Path) {
    match fs::remove_file(path).await {
        Ok(()) => {}

        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}

        Err(error) => {
            tracing::error!(
                path = %path.display(),
                error = %error,
                "remove local file failed"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> FileStorageConfig {
        FileStorageConfig {
            root_dir: PathBuf::from("/var/lib/medi-stream/uploads"),
            public_prefix: "/uploads".to_string(),
            max_size_bytes: 1024,
        }
    }

    #[test]
    fn public_file_path_maps_managed_url_inside_storage_root() {
        let path = public_file_path("/uploads/avatar.jpg", &test_config())
            .expect("managed URL should be valid")
            .expect("managed URL should resolve");

        assert_eq!(
            path,
            PathBuf::from("/var/lib/medi-stream/uploads/avatar.jpg")
        );
    }

    #[test]
    fn public_file_path_rejects_parent_directory_traversal() {
        let result = public_file_path("/uploads/../../etc/passwd", &test_config());

        assert!(matches!(result, Err(AppError::Internal(_))));
    }

    #[test]
    fn public_file_path_ignores_external_url() {
        let path = public_file_path("https://example.com/avatar.jpg", &test_config())
            .expect("external URL should not be treated as an error");

        assert!(path.is_none());
    }
}
