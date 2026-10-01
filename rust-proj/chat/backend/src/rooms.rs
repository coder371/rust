//! إدارة الغرف عبر REST.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    models::{ChatMessage, RoomSummary},
    state::AppState,
};

const MAX_ROOM_NAME: usize = 48;

#[derive(Deserialize)]
pub struct CreateRoom {
    pub name: String,
}

pub async fn list_rooms(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Json<Vec<RoomSummary>> {
    Json(state.room_summaries())
}

pub async fn create_room(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRoom>,
) -> AppResult<Json<RoomSummary>> {
    let name = body.name.trim().to_string();

    if name.is_empty() || name.chars().count() > MAX_ROOM_NAME {
        return Err(AppError::BadRequest(format!(
            "اسم الغرفة لازم يكون بين 1 و {MAX_ROOM_NAME} حرف"
        )));
    }

    Ok(Json(state.create_room(name, user.username)?))
}

pub async fn room_messages(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<String>,
) -> AppResult<Json<Vec<ChatMessage>>> {
    let room = state
        .room(&room_id)
        .ok_or_else(|| AppError::NotFound("الغرفة مش موجودة".to_string()))?;

    Ok(Json(room.history_snapshot()))
}
