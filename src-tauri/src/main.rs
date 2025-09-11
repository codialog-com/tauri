#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cdp;
mod tagui;
mod llm;
mod logging;

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    extract::Query,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error, debug};
use logging::LogManager;
use std::collections::HashMap;

#[derive(Clone)]
struct AppState {
    webview_url: Arc<Mutex<String>>,
    log_manager: Arc<LogManager>,
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

#[derive(Serialize, Deserialize)]
struct LogQuery {
    log_type: Option<String>, // "app", "error", "debug", "tagui"
    lines: Option<usize>,     // liczba linii do pobrania
}

#[derive(Serialize, Deserialize)]
struct LogResponse {
    success: bool,
    logs: Option<Vec<String>>,
    stats: Option<serde_json::Value>,
    error: Option<String>,
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

// Endpoint do pobierania logów
async fn get_logs(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Json<LogResponse> {
    info!("Getting logs with params: {:?}", params);
    
    let log_type = params.get("log_type").cloned().unwrap_or_else(|| "app".to_string());
    let lines = params.get("lines")
        .and_then(|s| s.parse::<usize>().ok());
    
    match state.log_manager.read_logs(&log_type, lines) {
        Ok(logs) => {
            info!("Successfully retrieved {} log lines for type: {}", logs.len(), log_type);
            Json(LogResponse {
                success: true,
                logs: Some(logs),
                stats: None,
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to read logs: {}", e);
            Json(LogResponse {
                success: false,
                logs: None,
                stats: None,
                error: Some(format!("Failed to read logs: {}", e)),
            })
        }
    }
}

// Endpoint do pobierania statystyk logów
async fn get_log_stats(
    State(state): State<AppState>,
) -> Json<LogResponse> {
    info!("Getting log statistics");
    
    match state.log_manager.get_log_stats() {
        Ok(stats) => {
            info!("Successfully retrieved log statistics");
            Json(LogResponse {
                success: true,
                logs: None,
                stats: Some(stats),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to get log stats: {}", e);
            Json(LogResponse {
                success: false,
                logs: None,
                stats: None,
                error: Some(format!("Failed to get log stats: {}", e)),
            })
        }
    }
}

// Endpoint do rotacji logów
async fn clear_logs(
    State(state): State<AppState>,
) -> Json<LogResponse> {
    info!("Starting log rotation");
    
    match state.log_manager.rotate_logs() {
        Ok(()) => {
            info!("Log rotation completed successfully");
            Json(LogResponse {
                success: true,
                logs: None,
                stats: None,
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to rotate logs: {}", e);
            Json(LogResponse {
                success: false,
                logs: None,
                stats: None,
                error: Some(format!("Failed to rotate logs: {}", e)),
            })
        }
    }
}

#[tauri::command]
async fn load_url(url: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    info!("Loading URL: {}", url);
    let mut webview_url = state.webview_url.lock().await;
    *webview_url = url;
    Ok(())
}

fn main() {
    // Initialize advanced logging system
    let log_manager = Arc::new(LogManager::new("logs"));
    
    if let Err(e) = log_manager.init_logging() {
        eprintln!("Failed to initialize logging system: {}", e);
        std::process::exit(1);
    }
    
    info!("🚀 Starting Codialog application...");
    info!("Advanced logging system initialized");
    
    let app_state = AppState {
        webview_url: Arc::new(Mutex::new(String::new())),
        log_manager: log_manager.clone(),
    };

    // Stwórz Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Uruchom serwer HTTP w tle
    let state_clone = app_state.clone();
    rt.spawn(async move {
        let app = Router::new()
            .route("/health", get(health))
            .route("/dsl/generate", post(generate_dsl))
            .route("/rpa/run", post(run_tagui))
            .route("/page/analyze", get(analyze_page))
            .route("/logs", get(get_logs))
            .route("/logs/stats", get(get_log_stats))
            .route("/logs/clear", post(clear_logs))
            .with_state(state_clone);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
            .await
            .expect("Failed to bind to port 4000");
        
        info!("HTTP server starting on http://127.0.0.1:4000");
        axum::serve(listener, app).await.expect("Failed to start HTTP server");
    });

    // Initialize TagUI if not present
    rt.spawn(async {
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
