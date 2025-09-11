use std::path::Path;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Result as IoResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub module: Option<String>,
}

pub struct LogManager {
    log_dir: String,
}

impl LogManager {
    pub fn new(log_dir: &str) -> Self {
        Self {
            log_dir: log_dir.to_string(),
        }
    }

    /// Inicjalizacja systemu logowania z zapisem do plików
    pub fn init_logging(&self) -> IoResult<()> {
        // Upewnij się, że katalog logs istnieje
        fs::create_dir_all(&self.log_dir)?;
        
        // Konfiguracja appender'ów dla różnych poziomów logów
        let app_file = RollingFileAppender::new(
            Rotation::DAILY,
            &self.log_dir,
            "app.log"
        );
        
        let error_file = RollingFileAppender::new(
            Rotation::DAILY,
            &self.log_dir,
            "error.log"
        );

        let debug_file = RollingFileAppender::new(
            Rotation::HOURLY,
            &self.log_dir,
            "debug.log"
        );

        // Filtry dla różnych poziomów
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        // Konfiguracja layerów
        let app_layer = tracing_subscriber::fmt::layer()
            .with_writer(app_file)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        let error_layer = tracing_subscriber::fmt::layer()
            .with_writer(error_file)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.level() <= &tracing::Level::WARN
            }));

        let debug_layer = tracing_subscriber::fmt::layer()
            .with_writer(debug_file)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.level() <= &tracing::Level::DEBUG
            }));

        let console_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .with_target(true);

        // Inicjalizacja subscriber
        tracing_subscriber::registry()
            .with(env_filter)
            .with(app_layer)
            .with(error_layer)
            .with(debug_layer)
            .with(console_layer)
            .init();

        info!("Sistema logowania został zainicjalizowany");
        info!("Logi zapisywane do katalogu: {}", self.log_dir);
        
        Ok(())
    }

    /// Odczyt logów z pliku
    pub fn read_logs(&self, log_type: &str, lines: Option<usize>) -> IoResult<Vec<String>> {
        let file_path = match log_type {
            "app" => format!("{}/app.log", self.log_dir),
            "error" => format!("{}/error.log", self.log_dir),
            "debug" => format!("{}/debug.log", self.log_dir),
            "tagui" => format!("{}/tagui.log", self.log_dir),
            _ => return Ok(vec!["Nieznany typ logu".to_string()]),
        };

        if !Path::new(&file_path).exists() {
            return Ok(vec![format!("Plik logu {} nie istnieje", file_path)]);
        }

        let content = fs::read_to_string(&file_path)?;
        let mut log_lines: Vec<String> = content
            .lines()
            .map(|line| line.to_string())
            .collect();

        // Zwróć ostatnie N linii jeśli określono
        if let Some(n) = lines {
            if log_lines.len() > n {
                log_lines = log_lines.split_off(log_lines.len() - n);
            }
        }

        Ok(log_lines)
    }

    /// Wyczyść stare logi
    pub fn rotate_logs(&self) -> IoResult<()> {
        info!("Rozpoczynanie rotacji logów...");
        
        // Rotacja jest automatyczna dzięki RollingFileAppender
        // Tutaj można dodać dodatkową logikę czyszczenia starych plików
        
        info!("Rotacja logów zakończona");
        Ok(())
    }

    /// Zapisz log TagUI do dedykowanego pliku
    pub fn log_tagui(&self, message: &str, success: bool) -> IoResult<()> {
        let tagui_log_path = format!("{}/tagui.log", self.log_dir);
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let status = if success { "SUCCESS" } else { "FAILED" };
        
        let log_line = format!("[{}] [{}] {}\n", timestamp, status, message);
        
        // Dodaj do pliku tagui.log
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tagui_log_path)?
            .write_all(log_line.as_bytes())?;

        // Loguj również do głównego systemu
        if success {
            info!(target: "tagui", "{}", message);
        } else {
            error!(target: "tagui", "{}", message);
        }
        
        Ok(())
    }

    /// Pobierz statystyki logów
    pub fn get_log_stats(&self) -> IoResult<serde_json::Value> {
        let mut stats = serde_json::Map::new();
        
        let log_files = ["app.log", "error.log", "debug.log", "tagui.log"];
        
        for file in &log_files {
            let path = format!("{}/{}", self.log_dir, file);
            
            if Path::new(&path).exists() {
                if let Ok(metadata) = fs::metadata(&path) {
                    let size = metadata.len();
                    let lines = fs::read_to_string(&path)
                        .map(|content| content.lines().count())
                        .unwrap_or(0);
                    
                    let mut file_stats = serde_json::Map::new();
                    file_stats.insert("size_bytes".to_string(), serde_json::Value::from(size));
                    file_stats.insert("lines".to_string(), serde_json::Value::from(lines));
                    file_stats.insert("exists".to_string(), serde_json::Value::from(true));
                    
                    stats.insert(file.replace(".log", ""), serde_json::Value::Object(file_stats));
                } else {
                    let mut file_stats = serde_json::Map::new();
                    file_stats.insert("exists".to_string(), serde_json::Value::from(false));
                    stats.insert(file.replace(".log", ""), serde_json::Value::Object(file_stats));
                }
            }
        }
        
        Ok(serde_json::Value::Object(stats))
    }
}

// Makra pomocnicze do logowania z kontekstem
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*);
    };
}

use std::io::Write;
