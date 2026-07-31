use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "config.json";
const KEY_SERVER_URL: &str = "remote_server_url";

/// Payload sent to the frontend when a previously-saved remote server is found.
#[derive(Clone, Serialize)]
pub struct RemoteServerPayload {
    pub url: String,
}

/// Tauri command: confirm this is NOT a desktop (embedded-server) build.
#[tauri::command]
fn is_desktop() -> bool {
    false
}

/// Tauri command: return the remote server URL saved in the native store, if any.
#[tauri::command]
fn get_remote_server_url(app: tauri::AppHandle) -> Option<String> {
    let store = app.store(STORE_FILE).ok()?;
    store
        .get(KEY_SERVER_URL)
        .and_then(|v| v.as_str().map(str::to_string))
}

/// Tauri command: persist the active remote server URL to the native store.
/// Call this from the frontend whenever the user adds or switches to a remote server.
#[tauri::command]
fn set_remote_server_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_SERVER_URL, serde_json::Value::String(url));
    store.save().map_err(|e| e.to_string())
}

#[tauri::mobile_entry_point]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            is_desktop,
            get_remote_server_url,
            set_remote_server_url,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // If a remote server URL was saved on a previous run, tell the frontend
            // so it can auto-select that server without waiting for user input.
            tauri::async_runtime::spawn(async move {
                if let Some(url) = get_remote_server_url(handle.clone()) {
                    tracing::info!("Restoring remote server: {}", url);
                    let _ = handle.emit("remote-server-ready", RemoteServerPayload { url });
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kanva mobile application");
}
