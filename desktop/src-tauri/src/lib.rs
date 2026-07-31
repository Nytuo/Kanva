mod updater;

use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

/// State managed by the Tauri app to track the embedded server.
pub struct EmbeddedServer {
    pub url: Mutex<Option<String>>,
    pub port: Mutex<u16>,
}

/// Payload sent to the frontend when the embedded server is ready.
#[derive(Clone, Serialize)]
pub struct ServerReadyPayload {
    pub url: String,
    pub port: u16,
}

/// Tauri command: get the embedded server URL.
/// Returns None if the server hasn't started yet.
#[tauri::command]
fn get_embedded_server_url(state: tauri::State<'_, EmbeddedServer>) -> Option<String> {
    state.url.lock().unwrap().clone()
}

/// Tauri command: check if we're running in desktop mode (always true here).
#[tauri::command]
fn is_desktop() -> bool {
    true
}

/// Start the embedded Kanva server on a random available port.
/// Returns the URL or an error message.
pub async fn start_embedded_server(data_dir: &str) -> Result<(String, u16), String> {
    // Find an available port
    let port = portpicker::pick_unused_port().ok_or("No available port")?;

    let mut config = kanva_server::Config::standalone(port, data_dir);

    // Persist the JWT secret so tokens remain valid across app restarts.
    // Config::standalone() generates a new random secret each call; we
    // override it with the one saved on first launch.
    let secret_path = std::path::Path::new(data_dir).join(".jwt_secret");
    match std::fs::read_to_string(&secret_path) {
        Ok(saved) => {
            let saved = saved.trim().to_string();
            if !saved.is_empty() {
                config.jwt_secret = saved;
            } else {
                // File exists but empty — save the newly generated secret
                let _ = std::fs::write(&secret_path, &config.jwt_secret);
            }
        }
        Err(_) => {
            // First launch — persist the generated secret for future restarts
            let _ = std::fs::write(&secret_path, &config.jwt_secret);
        }
    }

    // Ensure data directories exist
    std::fs::create_dir_all(&config.upload_dir).map_err(|e| e.to_string())?;

    let url = kanva_server::start_server(config)
        .await
        .map_err(|e| format!("Failed to start embedded server: {}", e))?;

    Ok((url, port))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(EmbeddedServer {
            url: Mutex::new(None),
            port: Mutex::new(0),
        })
        .invoke_handler(tauri::generate_handler![
            get_embedded_server_url,
            is_desktop,
            updater::check_for_update,
            updater::install_update,
            updater::open_releases_page,
            updater::restart_app,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Check for an app update shortly after launch and notify the
            // frontend if one is available (UpdaterModal listens for this).
            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                match updater::check_for_update(update_handle.clone()).await {
                    Ok(Some(info)) => {
                        tracing::info!("Update available: {}", info.version);
                        let _ = update_handle.emit("updater-update-available", &info);
                    }
                    Ok(None) => tracing::info!("Kanva desktop is up to date."),
                    Err(e) => tracing::warn!("Update check failed: {}", e),
                }
            });

            // Resolve data directory for embedded server
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");
            let data_dir_str = data_dir.to_string_lossy().to_string();

            // Spawn the embedded server in a background task
            tauri::async_runtime::spawn(async move {
                tracing::info!("Starting embedded Kanva server...");

                match start_embedded_server(&data_dir_str).await {
                    Ok((url, port)) => {
                        tracing::info!("Embedded server running at {} (port {})", url, port);

                        // Store in app state
                        let state = handle.state::<EmbeddedServer>();
                        *state.url.lock().unwrap() = Some(url.clone());
                        *state.port.lock().unwrap() = port;

                        // Emit event to frontend
                        let _ = handle.emit(
                            "embedded-server-ready",
                            ServerReadyPayload { url, port },
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to start embedded server: {}", e);
                        let _ = handle.emit("embedded-server-error", e);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kanva desktop application");
}
