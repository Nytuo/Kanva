use axum::{http, Router, Json, extract::State};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::{AppState, ServerInfo};

pub(crate) mod auth;
pub(crate) mod boards;
pub(crate) mod cards;
pub(crate) mod lists;
pub(crate) mod notes;
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
        let origin = state.config.cors_origin.parse::<http::HeaderValue>().unwrap_or_else(|_| {
            tracing::warn!(
                "CORS_ORIGIN '{}' is not a valid header value — falling back to http://localhost:3000",
                state.config.cors_origin
            );
            http::HeaderValue::from_static("http://localhost:3000")
        });
        CorsLayer::new()
            .allow_origin(origin)
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

    let static_dir = state.config.static_dir.clone();
    let upload_dir = state.config.upload_dir.clone();

    let mut router = Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/users", users::router())
        .nest("/api/boards", boards::router())
        .nest("/api/lists", lists::router())
        .nest("/api/cards", cards::router())
        .nest("/api/notes", notes::router())
        .nest("/api/calendar", calendar::router())
        .nest("/api/integrations", integrations::router());

    // Only mount teams routes when not in standalone mode
    if !state.config.standalone_mode {
        router = router.nest("/api/teams", teams::router());
    }

    router = router
        .route("/api/server-info", axum::routing::get(server_info))
        .route("/ws", axum::routing::get(crate::ws::ws_handler))
        .route("/health", axum::routing::get(health_check));

    // Serve the web SPA and user uploads from the same origin as the API.
    // Only enabled when a static dir is configured (embedded desktop app).
    // Explicit API/WS routes above take precedence; everything else falls
    // through to the SPA's index.html so client-side routing works on reload.
    if let Some(dir) = static_dir {
        let index = std::path::Path::new(&dir).join("index.html");
        router = router
            .nest_service("/uploads", ServeDir::new(&upload_dir))
            .fallback_service(ServeDir::new(&dir).fallback(ServeFile::new(index)));
    }

    router
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
