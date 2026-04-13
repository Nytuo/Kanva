use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,       // user_id
    pub email: String,
    pub exp: i64,        // expiration timestamp
    pub iat: i64,        // issued at timestamp
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = <AppState as FromRef<S>>::from_ref(state);

        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(app_state.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
            email: token_data.claims.email,
        })
    }
}

pub fn create_token(
    user_id: Uuid,
    email: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp: (now + chrono::Duration::hours(expiry_hours)).timestamp(),
        iat: now.timestamp(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("Token creation failed: {}", e)))
}
