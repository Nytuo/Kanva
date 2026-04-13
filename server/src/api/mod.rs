use axum::{http, Router, Json, extract::State};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::compression::CompressionLayer;

use crate::{AppState, ServerInfo};

pub(crate) mod auth;
pub(crate) mod boards;
pub(crate) mod cards;
pub(crate) mod lists;
pub(crate) mod teams;
pub(crate) mod calendar;
pub(crate) mod integrations;
pub(crate) mod users;

pub fn create_router(state: AppState) -> Router {
    let cors = if state.config.standalone_mode {
        // In standalone mode allow any origin (local client only)
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
            ])
    } else {
        CorsLayer::new()
            .allow_origin(state.config.cors_origin.parse::<http::HeaderValue>().unwrap())
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
            ])
            .allow_credentials(true)
    };

    let mut router = Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/users", users::router())
        .nest("/api/boards", boards::router())
        .nest("/api/lists", lists::router())
        .nest("/api/cards", cards::router())
        .nest("/api/calendar", calendar::router())
        .nest("/api/integrations", integrations::router());

    // Only mount teams routes when not in standalone mode
    if !state.config.standalone_mode {
        router = router.nest("/api/teams", teams::router());
    }

    router
        .route("/api/server-info", axum::routing::get(server_info))
        .route("/ws", axum::routing::get(crate::ws::ws_handler))
        .route("/health", axum::routing::get(health_check))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn server_info(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(ServerInfo::from_config(&state.config))
}
