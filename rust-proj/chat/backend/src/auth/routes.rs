//! تسجيل الدخول وإنشاء الحساب.

use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{jwt, password, AuthUser},
    error::{AppError, AppResult},
    state::AppState,
};

const MIN_USERNAME: usize = 3;
const MAX_USERNAME: usize = 32;
const MIN_PASSWORD: usize = 8;

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub username: String,
    pub expires_in: u64,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub username: String,
    pub created_at: Option<u64>,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Credentials>,
) -> AppResult<Json<AuthResponse>> {
    let username = validate(&body)?;

    let password_hash = password::hash_password(body.password).await?;
    state.insert_user(username.clone(), password_hash)?;

    Ok(Json(issue_token(&state, username)?))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Credentials>,
) -> AppResult<Json<AuthResponse>> {
    let username = body.username.trim().to_string();

    let stored_hash = state.password_hash_of(&username);

    // نفس الرسالة سواء الاسم مش موجود أو الباسورد غلط — عشان ما نكشفش
    // لحد بيجرّب أسماء إن الاسم ده موجود فعلاً.
    let invalid = || AppError::Unauthorized("اسم المستخدم أو الباسورد غلط".to_string());

    let Some(stored_hash) = stored_hash else {
        return Err(invalid());
    };

    if !password::verify_password(body.password, stored_hash).await? {
        return Err(invalid());
    }

    Ok(Json(issue_token(&state, username)?))
}

pub async fn me(user: AuthUser, State(state): State<Arc<AppState>>) -> Json<MeResponse> {
    let created_at = state.user_created_at(&user.username);

    Json(MeResponse {
        username: user.username,
        created_at,
    })
}

fn issue_token(state: &AppState, username: String) -> AppResult<AuthResponse> {
    let token = jwt::create_token(
        &username,
        state.config.jwt_secret.as_bytes(),
        state.config.jwt_ttl_seconds,
    )?;

    Ok(AuthResponse {
        token,
        username,
        expires_in: state.config.jwt_ttl_seconds,
    })
}

/// بيتأكد من صحة البيانات وبيرجّع الاسم بعد التنظيف.
fn validate(body: &Credentials) -> AppResult<String> {
    let username = body.username.trim().to_string();

    if username.chars().count() < MIN_USERNAME || username.chars().count() > MAX_USERNAME {
        return Err(AppError::BadRequest(format!(
            "اسم المستخدم لازم يكون بين {MIN_USERNAME} و {MAX_USERNAME} حرف"
        )));
    }

    if username.chars().any(char::is_whitespace) {
        return Err(AppError::BadRequest(
            "اسم المستخدم ماينفعش يحتوي مسافات".to_string(),
        ));
    }

    if body.password.chars().count() < MIN_PASSWORD {
        return Err(AppError::BadRequest(format!(
            "الباسورد لازم {MIN_PASSWORD} حروف على الأقل"
        )));
    }

    Ok(username)
}
