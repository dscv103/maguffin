//! Maguffin: A Rust-based Desktop Git Client
//!
//! This application provides a Tower-style PR dashboard and Graphite-style
//! stacked PR workflow for GitHub repositories.

// Module declarations
pub mod cache;
pub mod commands;
pub mod config;
pub mod domain;
pub mod error;
pub mod git;
pub mod github;
pub mod keyring;
pub mod provider;

use commands::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize application logging.
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "maguffin_app_lib=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tracing::info!("Starting Maguffin application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(commands::generate_handlers())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
