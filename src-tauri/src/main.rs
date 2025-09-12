#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cdp;
mod tagui;
mod llm;
mod logging;
mod bitwarden;
mod session;

#[cfg(test)]
mod tests;

use axum::{
    routing::{get, post},
    Router,
    extract::{Json, State, Query},
    response::Json as ResponseJson,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use tracing::{info, error, warn, debug, instrument, span, Level};
use logging::LogManager;
use bitwarden::{BitwardenManager, BitwardenCredential};
use session::{SessionManager, UserSession, UserData};
use std::collections::HashMap;
use sqlx::PgPool;
use redis::Client as RedisClient;
use anyhow::{Result, Context};

#[derive(Clone)]
struct AppState {
    webview_url: Arc<Mutex<String>>,
    log_manager: Arc<LogManager>,
    bitwarden_manager: Arc<Mutex<BitwardenManager>>,
    session_manager: Arc<SessionManager>,
    db_pool: PgPool,
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

#[derive(Serialize, Deserialize)]
struct BitwardenLoginRequest {
    email: String,
    master_password: String,
}

#[derive(Serialize, Deserialize)]
struct BitwardenUnlockRequest {
    master_password: String,
}

#[derive(Serialize, Deserialize)]
struct SessionRequest {
    user_id: String,
    user_data: UserData,
}

#[derive(Serialize, Deserialize)]
struct SessionResponse {
    success: bool,
    session: Option<UserSession>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CredentialsResponse {
    success: bool,
    credentials: Option<Vec<BitwardenCredential>>,
    error: Option<String>,
}

// Endpoint do generowania DSL z wsparciem cache'owania
#[instrument(skip(state, payload), fields(html_length = payload.html.len(), user_data_fields = payload.user_data.as_object().map(|obj| obj.len()).unwrap_or(0)))]
async fn generate_dsl(
    State(state): State<AppState>,
    Json(payload): Json<DslRequest>,
) -> Json<DslResponse> {
    let span = span!(Level::INFO, "generate_dsl_endpoint");
    let _enter = span.enter();
    
    info!(
        html_length = payload.html.len(),
        user_data_fields = payload.user_data.as_object().map(|obj| obj.len()).unwrap_or(0),
        "Starting DSL script generation with caching"
    );
    
    debug!("HTML preview: {}", &payload.html.chars().take(200).collect::<String>());
    debug!("User data keys: {:?}", payload.user_data.as_object().map(|obj| obj.keys().collect::<Vec<_>>()).unwrap_or_default());
    
    let start_time = std::time::Instant::now();
    
    // Use enhanced DSL generation with database caching
    let script = llm::generate_dsl_script_with_cache(
        &payload.html, 
        &payload.user_data, 
        Some(&state.db_pool)
    ).await;
    
    let generation_time = start_time.elapsed();
    
    info!(
        script_length = script.len(),
        generation_time_ms = generation_time.as_millis(),
        "DSL script generation completed successfully"
    );
    
    debug!("Generated script preview: {}", &script.chars().take(300).collect::<String>());
    
    // Log to database for analytics
    if let Err(e) = logging::log_system_event(
        &state.db_pool,
        "dsl_generator", 
        "info",
        &serde_json::json!({
            "operation": "dsl_generation",
            "html_length": payload.html.len(),
            "script_length": script.len(),
            "generation_time_ms": generation_time.as_millis(),
            "user_data_fields": payload.user_data.as_object().map(|obj| obj.len()).unwrap_or(0)
        })
    ).await {
        warn!("Failed to log DSL generation event: {}", e);
    }
    
    Json(DslResponse { script })
}

// Endpoint do uruchamiania skryptu TagUI
#[instrument(skip(payload), fields(script_length = payload.script.len()))]
async fn run_tagui(
    Json(payload): Json<RunScriptRequest>,
) -> Json<serde_json::Value> {
    let span = span!(Level::INFO, "run_tagui_endpoint");
    let _enter = span.enter();
    
    info!(
        script_length = payload.script.len(),
        "Starting TagUI script execution"
    );
    
    debug!("TagUI script preview: {}", &payload.script.chars().take(500).collect::<String>());
    
    let start_time = std::time::Instant::now();
    let result = tagui::execute_script(&payload.script).await;
    let execution_time = start_time.elapsed();
    
    match result {
        true => {
            info!(
                execution_time_ms = execution_time.as_millis(),
                "TagUI script executed successfully"
            );
        }
        false => {
            warn!(
                execution_time_ms = execution_time.as_millis(),
                "TagUI script execution failed"
            );
        }
    }
    
    debug!("TagUI execution result: {}", result);
    
    Json(serde_json::json!({ 
        "success": result,
        "execution_time_ms": execution_time.as_millis(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Endpoint do analizy strony przez CDP
#[instrument(skip(state))]
async fn analyze_page(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let span = span!(Level::INFO, "analyze_page_endpoint");
    let _enter = span.enter();
    
    info!("Starting page analysis with CDP");
    
    let start_time = std::time::Instant::now();
    let url = state.webview_url.lock().await;
    
    debug!("Current webview URL: {}", *url);
    
    let html = match cdp::get_page_html(&url).await {
        Ok(content) => {
            let analysis_time = start_time.elapsed();
            info!(
                html_length = content.len(),
                analysis_time_ms = analysis_time.as_millis(),
                url = %*url,
                "Page analysis completed successfully"
            );
            
            debug!("HTML content preview: {}", &content.chars().take(200).collect::<String>());
            content
        }
        Err(e) => {
            let analysis_time = start_time.elapsed();
            error!(
                analysis_time_ms = analysis_time.as_millis(),
                url = %*url,
                error = %e,
                "Page analysis failed"
            );
            String::new()
        }
    };
    
    Json(serde_json::json!({ 
        "html": html,
        "url": *url,
        "analysis_time_ms": start_time.elapsed().as_millis(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
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

// Endpoint do logowania się do Bitwarden
#[axum::debug_handler]
async fn bitwarden_login(
    Json(payload): Json<BitwardenLoginRequest>,
    State(state): State<AppState>,
) -> ResponseJson<SessionResponse> {
    info!("Bitwarden login attempt for user: {}", payload.email);
    
    let mut bitwarden = state.bitwarden_manager.lock().await;
    
    match bitwarden.login(&payload.email, &payload.master_password).await {
        Ok(()) => {
            info!("Bitwarden login successful for: {}", payload.email);
            
            // Utwórz sesję użytkownika
            let user_data = UserData::default();
            match state.session_manager.create_session(&payload.email, user_data).await {
                Ok(session) => {
                    ResponseJson(SessionResponse {
                        success: true,
                        session: Some(session),
                        error: None,
                    })
                }
                Err(e) => {
                    error!("Failed to create session: {}", e);
                    ResponseJson(SessionResponse {
                        success: false,
                        session: None,
                        error: Some(format!("Failed to create session: {}", e)),
                    })
                }
            }
        }
        Err(e) => {
            error!("Bitwarden login failed: {}", e);
            ResponseJson(SessionResponse {
                success: false,
                session: None,
                error: Some(format!("Bitwarden login failed: {}", e)),
            })
        }
    }
}

// Endpoint do odblokowywania Bitwarden vault
#[axum::debug_handler]
async fn bitwarden_unlock(
    Json(payload): Json<BitwardenUnlockRequest>,
    State(state): State<AppState>,
) -> ResponseJson<serde_json::Value> {
    info!("Bitwarden vault unlock attempt");
    
    let mut bitwarden = state.bitwarden_manager.lock().await;
    
    match bitwarden.unlock(&payload.master_password).await {
        Ok(()) => {
            info!("Bitwarden vault unlocked successfully");
            ResponseJson(serde_json::json!({
                "success": true,
                "message": "Vault unlocked successfully"
            }))
        }
        Err(e) => {
            error!("Failed to unlock Bitwarden vault: {}", e);
            ResponseJson(serde_json::json!({
                "success": false,
                "error": format!("Failed to unlock vault: {}", e)
            }))
        }
    }
}

// Endpoint do pobierania wszystkich danych logowania
async fn get_credentials(
    State(state): State<AppState>,
) -> Json<CredentialsResponse> {
    info!("Retrieving all credentials from Bitwarden");
    
    let bitwarden = state.bitwarden_manager.lock().await;
    
    match bitwarden.get_all_credentials().await {
        Ok(credentials) => {
            info!("Retrieved {} credentials", credentials.len());
            Json(CredentialsResponse {
                success: true,
                credentials: Some(credentials),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to retrieve credentials: {}", e);
            Json(CredentialsResponse {
                success: false,
                credentials: None,
                error: Some(format!("Failed to retrieve credentials: {}", e)),
            })
        }
    }
}

// Endpoint do pobierania danych logowania dla konkretnej strony
async fn get_credentials_for_url(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Json<CredentialsResponse> {
    let url = params.get("url").cloned().unwrap_or_default();
    info!("Retrieving credentials for URL: {}", url);
    
    let bitwarden = state.bitwarden_manager.lock().await;
    
    match bitwarden.get_credentials_for_url(&url).await {
        Ok(credentials) => {
            info!("Found {} credentials for URL: {}", credentials.len(), url);
            Json(CredentialsResponse {
                success: true,
                credentials: Some(credentials),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to retrieve credentials for URL: {}", e);
            Json(CredentialsResponse {
                success: false,
                credentials: None,
                error: Some(format!("Failed to retrieve credentials: {}", e)),
            })
        }
    }
}

// Endpoint do tworzenia/aktualizacji sesji użytkownika
#[axum::debug_handler]
async fn create_session(
    Json(payload): Json<SessionRequest>,
    State(state): State<AppState>,
) -> ResponseJson<SessionResponse> {
    info!("Creating session for user: {}", payload.user_id);
    
    match state.session_manager.create_session(&payload.user_id, payload.user_data).await {
        Ok(session) => {
            info!("Session created successfully: {}", session.session_id);
            ResponseJson(SessionResponse {
                success: true,
                session: Some(session),
                error: None,
            })
        }
        Err(e) => {
            error!("Failed to create session: {}", e);
            ResponseJson(SessionResponse {
                success: false,
                session: None,
                error: Some(format!("Failed to create session: {}", e)),
            })
        }
    }
}

// Endpoint do pobierania sesji
async fn get_session(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Json<SessionResponse> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    info!("Retrieving session: {}", session_id);
    
    match state.session_manager.get_session(&session_id).await {
        Ok(Some(session)) => {
            info!("Session found: {}", session_id);
            Json(SessionResponse {
                success: true,
                session: Some(session),
                error: None,
            })
        }
        Ok(None) => {
            warn!("Session not found: {}", session_id);
            Json(SessionResponse {
                success: false,
                session: None,
                error: Some("Session not found or expired".to_string()),
            })
        }
        Err(e) => {
            error!("Failed to retrieve session: {}", e);
            Json(SessionResponse {
                success: false,
                session: None,
                error: Some(format!("Failed to retrieve session: {}", e)),
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

async fn initialize_database() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://codialog:password@localhost:5432/codialog".to_string());
    
    info!("Connecting to database: {}", database_url);
    
    let pool = PgPool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;
    
    // Database migrations would be handled by Docker initialization
    // or manual migration scripts for production deployment
    info!("Database connection established, migrations handled externally");
    
    info!("Database initialized successfully");
    Ok(pool)
}

fn main() {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize advanced logging system
    let log_manager = Arc::new(LogManager::new("logs"));
    
    if let Err(e) = log_manager.init_logging() {
        eprintln!("Failed to initialize logging system: {}", e);
        std::process::exit(1);
    }
    
    info!("🚀 Starting Codialog application with Bitwarden integration...");
    info!("Advanced logging system initialized");
    
    // Stwórz Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Initialize database and Redis connections
    let (db_pool, redis_client, bitwarden_manager, session_manager) = rt.block_on(async {
        // Initialize database
        let db_pool = initialize_database().await
            .expect("Failed to initialize database");
        
        // Initialize Redis
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let redis_client = RedisClient::open(redis_url)
            .expect("Failed to create Redis client");
        
        // Initialize Bitwarden manager
        let bitwarden_server = std::env::var("BITWARDEN_SERVER")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let bitwarden_cli_server = std::env::var("BITWARDEN_CLI_SERVER")
            .unwrap_or_else(|_| "http://localhost:8087".to_string());
            
        let mut bitwarden_manager = BitwardenManager::new(bitwarden_server, bitwarden_cli_server);
        if let Err(e) = bitwarden_manager.initialize().await {
            warn!("Failed to initialize Bitwarden manager: {}", e);
        }
        
        // Initialize session manager
        let session_manager = SessionManager::new(db_pool.clone(), redis_client.clone());
        if let Err(e) = session_manager.initialize().await {
            error!("Failed to initialize session manager: {}", e);
            std::process::exit(1);
        }
        
        (db_pool, redis_client, bitwarden_manager, session_manager)
    });
    
    let app_state = AppState {
        webview_url: Arc::new(Mutex::new(String::new())),
        log_manager: log_manager.clone(),
        bitwarden_manager: Arc::new(Mutex::new(bitwarden_manager)),
        session_manager: Arc::new(session_manager),
        db_pool,
    };

    // Uruchom serwer HTTP w tle
    let state_clone = app_state.clone();
    rt.spawn(async move {
        let app = Router::new()
            // Health and system endpoints
            .route("/health", get(health))
            // DSL and automation endpoints  
            .route("/dsl/generate", post(generate_dsl))
            .route("/rpa/run", post(run_tagui))
            .route("/page/analyze", get(analyze_page))
            // Logging endpoints
            .route("/logs", get(get_logs))
            .route("/logs/stats", get(get_log_stats))
            .route("/logs/clear", post(clear_logs))
            // Bitwarden endpoints
            .route("/bitwarden/login", post(bitwarden_login))
            .route("/bitwarden/unlock", post(bitwarden_unlock))
            .route("/bitwarden/credentials", get(get_credentials))
            .route("/bitwarden/credentials/url", get(get_credentials_for_url))
            // Session management endpoints
            .route("/session/create", post(create_session))
            .route("/session/get", get(get_session))
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
