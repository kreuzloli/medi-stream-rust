use crate::error::AppError;
use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

/// 项目统一 HTTP 客户端。
///
/// 为什么不直接在业务代码里 reqwest::Client::new()？
/// 1. Client 内部有连接池，应该复用。
/// 2. 统一超时，避免外部 API 卡死拖垮接口。
/// 3. 统一错误处理，微信、腾讯云调用失败时，日志和返回格式一致。
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// 创建 HTTP 客户端。
    ///
    /// timeout_seconds:
    /// - 控制整个请求的最大耗时。
    /// - 外部 API 不应该无限等，否则一个慢接口能把整个服务拖成树懒。
    pub fn new(timeout_seconds: u64) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()?;

        Ok(Self { client })
    }

    /// 发送 GET 请求，并把响应解析成 JSON。
    ///
    /// 适合：
    /// - 微信获取 access_token
    /// - 微信查询素材
    /// - 普通第三方查询接口
    pub async fn get_json<T>(&self, service: &str, url: &str) -> Result<T, AppError>
    where
        T: DeserializeOwned,
    {
        let request = self.client.get(url);
        self.send_json(service, request).await
    }

    /// 发送 POST JSON 请求，并把响应解析成 JSON。
    ///
    /// 适合：
    /// - 微信发送模板消息
    /// - 微信创建菜单
    /// - 腾讯云直播 API 的 JSON body
    pub async fn post_json<B, T>(&self, service: &str, url: &str, body: &B) -> Result<T, AppError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let request = self.client.post(url).json(body);
        self.send_json(service, request).await
    }

    /// 发送 POST Form 请求，并把响应解析成 JSON。
    ///
    /// 目前微信和腾讯云大多用 GET / JSON，
    /// 但保留 form 方法，后面接别的服务时不用再补。
    pub async fn post_form<B, T>(&self, service: &str, url: &str, form: &B) -> Result<T, AppError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let request = self.client.post(url).form(form);
        self.send_json(service, request).await
    }

    /// 发送已经组装好的请求，并把响应解析成 JSON。
    ///
    /// 这个方法是给高级场景用的：
    /// - 腾讯云 API 需要加签名 header
    /// - 某些接口需要特殊 header
    /// - 某些接口需要手动设置 query/body
    pub async fn send_json<T>(&self, service: &str, request: RequestBuilder) -> Result<T, AppError>
    where
        T: DeserializeOwned,
    {
        let response = request.send().await?;
        let status = response.status();

        // 先拿文本，因为失败时要把 body 放进日志/错误里。
        // 成功时再把文本反序列化成目标类型。
        let body = response.text().await?;

        if !status.is_success() {
            tracing::warn!(
                service = service,
                status = status.as_u16(),
                body = %body,
                "external api returned non-success status"
            );

            return Err(AppError::ExternalApi {
                service: service.to_string(),
                status: status.as_u16(),
                body,
            });
        }

        let data = serde_json::from_str::<T>(&body)?;

        Ok(data)
    }

    /// 暴露底层 reqwest::Client。
    ///
    /// 一般业务不建议直接用。
    /// 但腾讯云签名这种场景，可能需要先 client.post(...) 再自己加 header，
    /// 所以这里留一个口子。
    pub fn raw(&self) -> &Client {
        &self.client
    }
}
