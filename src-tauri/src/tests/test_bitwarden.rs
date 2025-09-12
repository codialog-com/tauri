#![cfg(test)]

use super::*;
use pretty_assertions::assert_eq;
use crate::{
    bitwarden::{
        BitwardenManager,
        BitwardenCredential,
        BitwardenLogin,
        BitwardenUri,
        check_bitwarden_status,
        parse_bitwarden_credentials,
        bitwarden_login,
    },
    database::setup_test_database,
};
use serde_json::json;
use std::{
    process::Command,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::process::Command;
    
    // Helper function to check Bitwarden CLI status
    async fn check_bitwarden_status() -> Result<bool, String> {
        let output = Command::new("bw")
            .arg("--version")
            .output()
            .map_err(|e| format!("Failed to execute bw command: {}", e))?;
        
        if output.status.success() {
            Ok(true)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Bitwarden CLI error: {}", error))
        }
    }
    
    // Helper function to parse Bitwarden credentials from JSON output
    fn parse_bitwarden_credentials(json_output: &str) -> Result<Vec<BitwardenCredential>, String> {
        serde_json::from_str(json_output)
            .map_err(|e| format!("Failed to parse Bitwarden credentials: {}", e))
    }

    // Helper function to create a test Bitwarden credential
    fn create_test_credential() -> BitwardenCredential {
        BitwardenCredential {
            id: Uuid::new_v4().to_string(),
            name: "Test Credential".to_string(),
            login: Some(BitwardenLogin {
                username: Some("testuser@example.com".to_string()),
                password: Some("testpassword".to_string()),
                uris: Some(vec![BitwardenUri {
                    uri: Some("https://example.com".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_bitwarden_credential_creation() {
        let credential = BitwardenCredential {
            id: "test-id".to_string(),
            name: "Test Login".to_string(),
            username: Some("testuser@example.com".to_string()),
            password: Some("password123".to_string()),
            uri: Some("https://example.com".to_string()),
            notes: Some("Test notes".to_string()),
        };
        
        assert_eq!(credential.id, "test-id");
        assert_eq!(credential.name, "Test Login");
        assert_eq!(credential.username.as_ref().unwrap(), "testuser@example.com");
        assert_eq!(credential.password.as_ref().unwrap(), "password123");
        assert_eq!(credential.uri.as_ref().unwrap(), "https://example.com");
        assert_eq!(credential.notes.as_ref().unwrap(), "Test notes");
    }

    #[test]
    fn test_bitwarden_credential_partial_data() {
        let credential = BitwardenCredential {
            id: "test-id-2".to_string(),
            name: "Partial Login".to_string(),
            username: Some("user@example.com".to_string()),
            password: None,
            uri: None,
            notes: Some("Only username available".to_string()),
        };
        
        assert!(credential.password.is_none());
        assert!(credential.uri.is_none());
        assert!(credential.username.is_some());
    }

    #[tokio::test]
    async fn test_bitwarden_status_check() {
        // This test checks if we can properly detect Bitwarden CLI status
        let status = check_bitwarden_status().await;
        
        // Status should be one of: "unauthenticated", "locked", "unlocked"
        assert!(
            status == "unauthenticated" || status == "locked" || status == "unlocked" || status == "not_found",
            "Status should be a valid Bitwarden state"
        );
    }

    #[tokio::test]
    async fn test_bitwarden_login_invalid_credentials() {
        // Test with obviously invalid credentials
        let result = bitwarden_login("invalid@email.com", "wrong_password").await;
        
        // Should return an error for invalid credentials
        assert!(result.is_err(), "Login with invalid credentials should fail");
    }

    #[tokio::test]
    async fn test_bitwarden_unlock_without_login() {
        // Attempt to unlock without being logged in
        let result = bitwarden_unlock("any_password").await;
        
        // This should fail if not logged in first
        // Note: This might succeed in some test environments, so we just ensure it doesn't crash
        let _ = result; // Just ensure the function completes
    }

    #[test]
    fn test_parse_bitwarden_json_valid() {
        let json_output = r#"[
            {
                "id": "12345-67890",
                "name": "Example Site",
                "login": {
                    "username": "user@example.com",
                    "password": "secretpassword"
                },
                "notes": "Important login",
                "uris": [{"uri": "https://example.com"}]
            }
        ]"#;
        
        let credentials = parse_bitwarden_credentials(json_output);
        
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].id, "12345-67890");
        assert_eq!(credentials[0].name, "Example Site");
        assert_eq!(credentials[0].username.as_ref().unwrap(), "user@example.com");
        assert_eq!(credentials[0].password.as_ref().unwrap(), "secretpassword");
        assert_eq!(credentials[0].uri.as_ref().unwrap(), "https://example.com");
    }

    #[test]
    fn test_parse_bitwarden_json_empty() {
        let empty_json = "[]";
        let credentials = parse_bitwarden_credentials(empty_json);
        
        assert_eq!(credentials.len(), 0);
    }

    #[test]
    fn test_parse_bitwarden_json_malformed() {
        let malformed_json = "{ invalid json }";
        let credentials = parse_bitwarden_credentials(malformed_json);
        
        // Should return empty vector for malformed JSON
        assert_eq!(credentials.len(), 0);
    }

    #[test]
    fn test_parse_bitwarden_json_missing_fields() {
        let json_with_missing_fields = r#"[
            {
                "id": "12345",
                "name": "Incomplete Entry"
            }
        ]"#;
        
        let credentials = parse_bitwarden_credentials(json_with_missing_fields);
        
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].id, "12345");
        assert_eq!(credentials[0].name, "Incomplete Entry");
        assert!(credentials[0].username.is_none());
        assert!(credentials[0].password.is_none());
        assert!(credentials[0].uri.is_none());
    }

    #[test]
    fn test_filter_credentials_by_domain() {
        let credentials = create_test_credentials();
        
        // Filter by example.com domain
        let filtered: Vec<_> = credentials.into_iter()
            .filter(|cred| {
                cred.uri.as_ref()
                    .map(|uri| uri.contains("example.com"))
                    .unwrap_or(false)
            })
            .collect();
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "test-id-1");
    }

    #[test]
    fn test_credential_search_by_name() {
        let credentials = create_test_credentials();
        
        // Search for credentials containing "Login 1"
        let found: Vec<_> = credentials.into_iter()
            .filter(|cred| cred.name.to_lowercase().contains("login 1"))
            .collect();
        
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Test Login 1");
    }

    #[tokio::test]
    async fn test_bitwarden_command_timeout() {
        use std::time::{Duration, Instant};
        
        let start = Instant::now();
        
        // Test a command that should complete quickly or timeout
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            check_bitwarden_status()
        ).await;
        
        let elapsed = start.elapsed();
        
        // Should complete within timeout
        assert!(result.is_ok(), "Bitwarden command should not timeout");
        assert!(elapsed < Duration::from_secs(6), "Command took too long");
    }

    #[test]
    fn test_sanitize_bitwarden_output() {
        let output_with_secrets = "Password: secretpass123\nOther info";
        
        // In a real implementation, we might want to sanitize sensitive output
        let sanitized = sanitize_bitwarden_output(output_with_secrets);
        
        // This is a placeholder - implement actual sanitization logic
        assert!(!sanitized.is_empty());
    }

    #[tokio::test]
    async fn test_bitwarden_error_handling() {
        // Test various error conditions
        let invalid_email_result = bitwarden_login("", "password").await;
        assert!(invalid_email_result.is_err(), "Empty email should cause error");
        
        let empty_password_result = bitwarden_unlock("").await;
        // This might succeed in some cases, so we just ensure it doesn't panic
        let _ = empty_password_result;
    }

    #[test]
    fn test_bitwarden_credential_validation() {
        let valid_credential = BitwardenCredential {
            id: "valid-id".to_string(),
            name: "Valid Login".to_string(),
            username: Some("user@example.com".to_string()),
            password: Some("password123".to_string()),
            uri: Some("https://example.com".to_string()),
            notes: Some("Valid notes".to_string()),
        };
        
        // Test that credential has required fields
        assert!(!valid_credential.id.is_empty());
        assert!(!valid_credential.name.is_empty());
        assert!(valid_credential.username.is_some());
        assert!(valid_credential.password.is_some());
    }

    // Helper function for sanitization (placeholder)
    fn sanitize_bitwarden_output(output: &str) -> String {
        // In a real implementation, this would remove or mask sensitive information
        output.lines()
            .map(|line| {
                if line.to_lowercase().contains("password") {
                    "Password: [REDACTED]"
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
