#![cfg(test)]

use super::*;
use crate::logging::*;
use pretty_assertions::assert_eq;
use serde_json::json;
use tokio::test;

mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tracing::{info, warn, error, debug};

    #[tokio::test]
    async fn test_log_file_creation() {
        let test_log_dir = "/tmp/test_codialog_logs";
        
        // Cleanup any existing test logs
        let _ = fs::remove_dir_all(test_log_dir);
        
        // Initialize logging with test directory
        let result = initialize_logging_system(Some(test_log_dir)).await;
        assert!(result.is_ok(), "Should initialize logging system successfully");
        
        // Check if log files are created
        assert!(Path::new(test_log_dir).exists(), "Log directory should be created");
        
        // Cleanup
        let _ = fs::remove_dir_all(test_log_dir);
    }

    #[tokio::test]
    async fn test_structured_logging() {
        let pool = setup_test_database().await;
        
        // Test different log levels
        info!("Test info message");
        warn!("Test warning message");
        error!("Test error message");
        debug!("Test debug message");
        
        // Test structured logging with context
        log_user_action(&pool, "test_user", "login_attempt", &json!({
            "ip_address": "127.0.0.1",
            "user_agent": "test_agent",
            "timestamp": "2023-01-01T12:00:00Z"
        })).await.expect("Should log user action");
        
        log_system_event(&pool, "bitwarden_unlock", "success", &json!({
            "vault_items": 10,
            "duration_ms": 150
        })).await.expect("Should log system event");
    }

    #[tokio::test]
    async fn test_log_retrieval() {
        let pool = setup_test_database().await;
        
        // Insert test log entries
        for i in 0..5 {
            log_user_action(&pool, "test_user", &format!("action_{}", i), &json!({
                "test_data": i
            })).await.expect("Should insert log entry");
        }
        
        // Retrieve logs
        let logs = get_application_logs(&pool, Some(10), None).await;
        assert!(logs.is_ok(), "Should retrieve logs successfully");
        
        let log_entries = logs.unwrap();
        assert!(!log_entries.is_empty(), "Should have log entries");
        assert!(log_entries.len() >= 5, "Should retrieve at least 5 entries");
    }

    #[tokio::test]
    async fn test_log_filtering() {
        let pool = setup_test_database().await;
        
        // Insert logs with different levels and components
        log_system_event(&pool, "bitwarden", "info", &json!({"message": "vault unlocked"})).await.unwrap();
        log_system_event(&pool, "database", "error", &json!({"message": "connection failed"})).await.unwrap();
        log_system_event(&pool, "dsl_generator", "debug", &json!({"message": "script generated"})).await.unwrap();
        
        // Test filtering by component
        let bitwarden_logs = get_logs_by_component(&pool, "bitwarden", Some(10)).await;
        assert!(bitwarden_logs.is_ok(), "Should filter logs by component");
        
        // Test filtering by level
        let error_logs = get_logs_by_level(&pool, "error", Some(10)).await;
        assert!(error_logs.is_ok(), "Should filter logs by level");
        
        let error_entries = error_logs.unwrap();
        assert!(!error_entries.is_empty(), "Should have error log entries");
    }

    #[tokio::test]
    async fn test_log_statistics() {
        let pool = setup_test_database().await;
        
        // Insert various log entries
        for level in &["info", "warn", "error", "debug"] {
            for i in 0..3 {
                log_system_event(&pool, "test_component", level, &json!({
                    "test_id": i,
                    "level": level
                })).await.unwrap();
            }
        }
        
        let stats = get_log_statistics(&pool).await;
        assert!(stats.is_ok(), "Should get log statistics successfully");
        
        let log_stats = stats.unwrap();
        assert!(log_stats.total_logs > 0, "Should have total log count");
        assert!(log_stats.error_count > 0, "Should have error count");
        assert!(log_stats.warning_count > 0, "Should have warning count");
    }

    #[test]
    fn test_log_entry_validation() {
        let valid_entry = LogEntry {
            id: 1,
            timestamp: chrono::Utc::now(),
            level: "info".to_string(),
            component: "test".to_string(),
            message: "Test message".to_string(),
            context: json!({"key": "value"}),
        };
        
        assert!(validate_log_entry(&valid_entry), "Valid log entry should pass validation");
        
        let invalid_entry = LogEntry {
            id: 2,
            timestamp: chrono::Utc::now(),
            level: "".to_string(), // Invalid empty level
            component: "test".to_string(),
            message: "".to_string(), // Invalid empty message
            context: json!({}),
        };
        
        assert!(!validate_log_entry(&invalid_entry), "Invalid log entry should fail validation");
    }

    #[tokio::test]
    async fn test_log_rotation() {
        let test_log_file = "/tmp/test_app.log";
        
        // Remove existing test file
        let _ = fs::remove_file(test_log_file);
        
        // Create log file and write entries
        for i in 0..1000 {
            write_log_to_file(test_log_file, &format!("Test log entry {}", i)).await.unwrap();
        }
        
        // Check if file exists and has content
        assert!(Path::new(test_log_file).exists(), "Log file should exist");
        
        let file_size = fs::metadata(test_log_file).unwrap().len();
        assert!(file_size > 0, "Log file should have content");
        
        // Test rotation
        rotate_log_file(test_log_file).await.expect("Should rotate log file");
        
        // Cleanup
        let _ = fs::remove_file(test_log_file);
        let _ = fs::remove_file(&format!("{}.1", test_log_file));
    }

    #[tokio::test]
    async fn test_performance_logging() {
        let pool = setup_test_database().await;
        
        let start_time = std::time::Instant::now();
        
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let duration = start_time.elapsed();
        
        // Log performance metrics
        log_performance_metric(&pool, "test_operation", duration.as_millis() as i64, &json!({
            "operation_type": "test",
            "success": true
        })).await.expect("Should log performance metric");
        
        // Retrieve performance logs
        let perf_logs = get_performance_logs(&pool, Some(10)).await;
        assert!(perf_logs.is_ok(), "Should retrieve performance logs");
        
        let logs = perf_logs.unwrap();
        assert!(!logs.is_empty(), "Should have performance log entries");
    }

    #[test]
    fn test_log_sanitization() {
        let sensitive_log = "User login attempt with password: secret123 and email: user@example.com";
        
        let sanitized = sanitize_log_message(sensitive_log);
        
        // Should remove sensitive information
        assert!(!sanitized.contains("secret123"), "Should remove password from logs");
        assert!(sanitized.contains("[REDACTED]") || sanitized.contains("***"), "Should mark redacted content");
        
        // Should keep non-sensitive information
        assert!(sanitized.contains("User login attempt"), "Should keep general information");
    }

    #[tokio::test]
    async fn test_concurrent_logging() {
        let pool = setup_test_database().await;
        
        // Simulate concurrent logging from multiple threads
        let mut handles = vec![];
        
        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                log_user_action(&pool_clone, &format!("user_{}", i), "concurrent_action", &json!({
                    "thread_id": i
                })).await
            });
            handles.push(handle);
        }
        
        // Wait for all logging operations to complete
        let results = futures::future::join_all(handles).await;
        
        // All logging operations should succeed
        for result in results {
            assert!(result.unwrap().is_ok(), "Concurrent logging should succeed");
        }
        
        // Verify all logs were written
        let logs = get_application_logs(&pool, Some(20), None).await.unwrap();
        assert!(logs.len() >= 10, "Should have logged all concurrent entries");
    }

    #[tokio::test]
    async fn test_log_cleanup() {
        let pool = setup_test_database().await;
        
        // Insert old log entries
        for i in 0..50 {
            log_system_event(&pool, "cleanup_test", "info", &json!({
                "entry": i,
                "created": "2022-01-01T00:00:00Z"
            })).await.unwrap();
        }
        
        // Run cleanup for logs older than 30 days
        let cleanup_result = cleanup_old_logs(&pool, 30).await;
        assert!(cleanup_result.is_ok(), "Should cleanup old logs successfully");
        
        let cleaned_count = cleanup_result.unwrap();
        assert!(cleaned_count >= 0, "Should return count of cleaned logs");
    }

    // Helper functions for logging tests
    
    #[derive(Debug)]
    struct LogEntry {
        id: i64,
        timestamp: chrono::DateTime<chrono::Utc>,
        level: String,
        component: String,
        message: String,
        context: serde_json::Value,
    }

    struct LogStatistics {
        total_logs: i64,
        error_count: i64,
        warning_count: i64,
        info_count: i64,
        debug_count: i64,
    }

    fn validate_log_entry(entry: &LogEntry) -> bool {
        !entry.level.is_empty() && 
        !entry.component.is_empty() && 
        !entry.message.is_empty() &&
        ["debug", "info", "warn", "error"].contains(&entry.level.as_str())
    }

    fn sanitize_log_message(message: &str) -> String {
        let mut sanitized = message.to_string();
        
        // Remove common sensitive patterns
        let sensitive_patterns = [
            (r"password:\s*\S+", "password: [REDACTED]"),
            (r"token:\s*\S+", "token: [REDACTED]"),
            (r"api[_-]?key:\s*\S+", "api_key: [REDACTED]"),
            (r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b", "[CARD_NUMBER]"),
        ];
        
        for (pattern, replacement) in &sensitive_patterns {
            sanitized = regex::Regex::new(pattern)
                .unwrap()
                .replace_all(&sanitized, *replacement)
                .to_string();
        }
        
        sanitized
    }
}
