use std::sync::Arc;

pub mod api;
pub mod middleware;
pub mod models;
pub mod services;
pub mod integrations;
pub mod ws;

pub use models::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::AnyPool,
    #[cfg(feature = "redis-support")]
    pub redis: Option<redis::aio::ConnectionManager>,
    pub config: Arc<Config>,
    pub ws_state: Arc<ws::WsState>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub cors_origin: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub gitlab_client_id: String,
    pub gitlab_client_secret: String,
    pub gitlab_redirect_uri: String,
    pub atlassian_client_id: String,
    pub atlassian_client_secret: String,
    pub atlassian_redirect_uri: String,
    pub max_upload_size_mb: usize,
    pub upload_dir: String,
    /// When true, the server runs in standalone/embedded mode.
    /// Teams features are disabled and CORS is permissive.
    pub standalone_mode: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()?,
            refresh_token_expiry_days: std::env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
            cors_origin: std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            github_client_id: std::env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
            github_client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
            github_redirect_uri: std::env::var("GITHUB_REDIRECT_URI").unwrap_or_default(),
            gitlab_client_id: std::env::var("GITLAB_CLIENT_ID").unwrap_or_default(),
            gitlab_client_secret: std::env::var("GITLAB_CLIENT_SECRET").unwrap_or_default(),
            gitlab_redirect_uri: std::env::var("GITLAB_REDIRECT_URI").unwrap_or_default(),
            atlassian_client_id: std::env::var("ATLASSIAN_CLIENT_ID").unwrap_or_default(),
            atlassian_client_secret: std::env::var("ATLASSIAN_CLIENT_SECRET").unwrap_or_default(),
            atlassian_redirect_uri: std::env::var("ATLASSIAN_REDIRECT_URI").unwrap_or_default(),
            max_upload_size_mb: std::env::var("MAX_UPLOAD_SIZE_MB")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
            upload_dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
            standalone_mode: std::env::var("STANDALONE_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()?,
        })
    }

    /// Create a minimal config for embedded/standalone mode.
    /// Uses SQLite-compatible defaults and generates a random JWT secret.
    pub fn standalone(port: u16, data_dir: &str) -> Self {
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: port,
            database_url: format!("sqlite:{}/kanva.db?mode=rwc", data_dir.replace('\\', "/")),
            redis_url: String::new(), // embedded mode won't use Redis
            jwt_secret: secret,
            jwt_expiry_hours: 720, // 30 days - local user doesn't need frequent re-auth
            refresh_token_expiry_days: 365,
            cors_origin: "*".to_string(),
            github_client_id: String::new(),
            github_client_secret: String::new(),
            github_redirect_uri: String::new(),
            gitlab_client_id: String::new(),
            gitlab_client_secret: String::new(),
            gitlab_redirect_uri: String::new(),
            atlassian_client_id: String::new(),
            atlassian_client_secret: String::new(),
            atlassian_redirect_uri: String::new(),
            max_upload_size_mb: 50,
            upload_dir: format!("{}/uploads", data_dir),
            standalone_mode: true,
        }
    }

    pub fn is_sqlite(&self) -> bool {
        self.database_url.starts_with("sqlite:")
    }
}

/// Start the server and return the address it's listening on.
/// This can be called from the standalone binary or embedded in Tauri.
pub async fn start_server(config: Config) -> anyhow::Result<String> {
    let addr = format!("{}:{}", config.server_host, config.server_port);

    tracing::info!("Starting Kanva server on {}", addr);
    if config.standalone_mode {
        tracing::info!("Running in STANDALONE mode (teams disabled, SQLite)");
    }

    // Install both drivers so AnyPool can pick based on URL prefix
    sqlx::any::install_default_drivers();

    // Database connection via AnyPool
    let is_sqlite = config.is_sqlite();
    let db = sqlx::any::AnyPoolOptions::new()
        .max_connections(if config.standalone_mode { 5 } else { 20 })
        // Every pooled connection needs this, not just the first one — set it via
        // after_connect rather than a one-off query on the pool.
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                if is_sqlite {
                    sqlx::query("PRAGMA foreign_keys = ON").execute(conn).await?;
                }
                Ok(())
            })
        })
        .connect(&config.database_url)
        .await?;

    // Run migrations — pick directory based on DB type
    if config.is_sqlite() {
        sqlx::migrate!("../migrations/sqlite").run(&db).await?;
    } else {
        sqlx::migrate!("../migrations/postgres").run(&db).await?;
    }
    tracing::info!("Database migrations applied");

    // Redis connection (optional, skip in standalone mode)
    #[cfg(feature = "redis-support")]
    let redis_conn: Option<redis::aio::ConnectionManager> = if config.redis_url.is_empty() {
        tracing::info!("Redis URL empty — running without Redis (JWT-only sessions)");
        None
    } else {
        match redis::Client::open(config.redis_url.as_str()) {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(conn) => {
                    tracing::info!("Connected to Redis");
                    Some(conn)
                }
                Err(e) => {
                    tracing::warn!("Redis connection failed ({}), continuing without it", e);
                    None
                }
            },
            Err(e) => {
                tracing::warn!("Redis client error ({}), continuing without it", e);
                None
            }
        }
    };

    let ws_state = Arc::new(ws::WsState::new());

    let state = AppState {
        db,
        #[cfg(feature = "redis-support")]
        redis: redis_conn,
        config: Arc::new(config),
        ws_state,
    };

    let app = api::create_router(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let local_addr = listener.local_addr()?;
    let listen_addr = format!("http://{}", local_addr);
    tracing::info!("Listening on {}", listen_addr);

    // Spawn the server in the background so this function returns
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Server error: {}", e);
        }
    });

    Ok(listen_addr)
}

impl AppState {
    /// Rewrite `?` placeholders to `$1`, `$2`, ... for PostgreSQL.
    /// When using SQLite, the SQL is returned unchanged.
    /// Use this for every query: `sqlx::query(state.db_sql("SELECT ... WHERE id = ?"))`
    pub fn q(&self, sql: &str) -> String {
        if self.config.is_sqlite() {
            sql.to_string()
        } else {
            rewrite_placeholders(sql)
        }
    }
}

/// Rewrite `?` placeholders to `$1`, `$2`, ... (PostgreSQL style).
/// Handles `?` inside single-quoted strings and `--` line comments correctly.
pub fn rewrite_placeholders(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len() + 16);
    let mut n = 1usize;
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // Single-quoted string literal — pass through unchanged
            '\'' => {
                result.push('\'');
                loop {
                    match chars.next() {
                        None => break,
                        Some('\'') => {
                            result.push('\'');
                            // Handle escaped quotes ''
                            if chars.peek() == Some(&'\'') {
                                result.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        Some(ch) => result.push(ch),
                    }
                }
            }
            // Line comment -- ... \n — pass through unchanged
            '-' if chars.peek() == Some(&'-') => {
                result.push('-');
                result.push(chars.next().unwrap()); // second '-'
                for ch in chars.by_ref() {
                    result.push(ch);
                    if ch == '\n' { break; }
                }
            }
            '?' => {
                use std::fmt::Write;
                write!(result, "${}", n).unwrap();
                n += 1;
            }
            other => result.push(other),
        }
    }
    result
}

/// Expose server capabilities for the client to query.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub standalone: bool,
    pub teams_enabled: bool,
}

impl ServerInfo {
    pub fn from_config(config: &Config) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            standalone: config.standalone_mode,
            teams_enabled: !config.standalone_mode,
        }
    }
}
