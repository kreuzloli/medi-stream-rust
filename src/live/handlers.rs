use crate::account::account_service;
use crate::common::jwt::CurrentUser;
use crate::error::AppError;
use crate::live::live_model::LiveWatchResp;
use crate::live::live_service;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;

/// 根据房间编码返回当前登录用户可使用的直播播放信息。
pub async fn watch_live_room(
    CurrentUser(claims): CurrentUser,
    Path(room_code): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<LiveWatchResp>, AppError> {
    let user_id = account_service::require_claim_user_id(&claims)?;
    tracing::info!(user_id, room_code = %room_code, "live watch request received");
    let response = live_service::get_live_watch(&state, &room_code).await?;
    tracing::info!(
        user_id,
        room_id = response.room.id,
        stream_id = response.stream.id,
        "live watch request completed"
    );
    Ok(Json(response))
}
