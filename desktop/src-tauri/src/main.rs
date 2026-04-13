#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    // Initialize tracing for the desktop app
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanva_desktop=debug,kanva_server=info".into()),
        )
        .init();

    kanva_desktop_lib::run();
}
