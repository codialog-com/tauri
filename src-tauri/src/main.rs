#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cdp;
mod tagui;
mod llm;

use tauri::Manager;
use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

#[derive(Clone)]
struct AppState {
    webview_url: Arc<Mutex<String>>,
}

#[derive(Serialize, Deserialize)]
struct DslRequest {
    html: String,
    user_data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct DslResponse {
    script: String,
}

#[derive(Serialize, Deserialize)]
struct RunScriptRequest {
    script: String,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    services: serde_json::Value,
}

// Endpoint do generowania DSL przez LLM
async fn generate_dsl(
    Json(payload): Json<DslRequest>,
) -> Json<DslResponse> {
    info!("Generating DSL script for form analysis");
    let script = llm::generate_dsl_script(&payload.html, &payload.user_data).await;
    Json(DslResponse { script })
}

// Endpoint do uruchamiania skryptu TagUI
async fn run_tagui(
    Json(payload): Json<RunScriptRequest>,
) -> Json<serde_json::Value> {
    info!("Executing TagUI script");
    let result = tagui::execute_script(&payload.script).await;
    Json(serde_json::json!({ "success": result }))
}

// Endpoint do analizy strony przez CDP
async fn analyze_page(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    info!("Analyzing current page with CDP");
    let url = state.webview_url.lock().await;
    let html = cdp::get_page_html(&url).await.unwrap_or_default();
    Json(serde_json::json!({ "html": html }))
}

// Health check endpoint
async fn health() -> Json<HealthResponse> {
    let services = serde_json::json!({
        "tagui": tagui::check_tagui_installed().await,
        "database": "not_implemented", 
        "redis": "not_implemented"
    });
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        services,
    })
}

#[tauri::command]
async fn load_url(url: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Loading URL: {}", url);
    let mut webview_url = state.webview_url.lock().await;
    *webview_url = url;
    Ok(())
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let app_state = AppState {
        webview_url: Arc::new(Mutex::new(String::new())),
    };

    // Uruchom serwer HTTP w tle
    let state_clone = app_state.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(health))
            .route("/dsl/generate", post(generate_dsl))
            .route("/rpa/run", post(run_tagui))
            .route("/page/analyze", get(analyze_page))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
            .await
            .expect("Failed to bind to port 4000");
        
        info!("HTTP server starting on http://127.0.0.1:4000");
        axum::serve(listener, app).await.expect("Failed to start HTTP server");
    });

    // Initialize TagUI if not present
    tokio::spawn(async {
        if !tagui::check_tagui_installed().await {
            info!("TagUI not found, installing...");
            if tagui::install_tagui() {
                info!("TagUI installed successfully");
            } else {
                error!("Failed to install TagUI");
            }
        }
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![load_url])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
