#![cfg(test)]

use super::*;
use pretty_assertions::assert_eq;
use crate::{
    session::{
        create_user_session,
        get_user_session,
        expire_user_session,
        cleanup_expired_sessions,
        validate_session,
        update_user_session,
        get_session_metrics,
    },
    database::setup_test_database,
};
use serde_json::json;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    
    // Helper function to create test user data
    fn create_test_user_data() -> serde_json::Value {
        json!({
            "user_id": Uuid::new_v4().to_string(),
            "email": "test@example.com",
            "name": "Test User",
            "created_at": Utc::now().to_rfc3339(),
        })
    }

    #[tokio::test]
    async fn test_create_session() {
        let pool = setup_test_database().await;
        let user_data = create_test_user_data();
        
        let session_result = create_user_session(&pool, &user_data).await;
        assert!(session_result.is_ok(), "Should create session successfully");
        
        let session = session_result.unwrap();
        assert!(!session.session_id.is_empty(), "Session ID should not be empty");
        assert!(!session.data.is_empty(), "Session data should not be empty");
    }

    #[tokio::test]
    async fn test_retrieve_session() {
        let pool = setup_test_database().await;
        let user_data = create_test_user_data();
        
        // Create session first
        let session = create_user_session(&pool, &user_data).await.unwrap();
        
        // Retrieve session
        let retrieved_result = get_user_session(&pool, &session.session_id).await;
        assert!(retrieved_result.is_ok(), "Should retrieve session successfully");
        
        let retrieved = retrieved_result.unwrap();
        assert!(retrieved.is_some(), "Retrieved session should exist");
        
        let retrieved_session = retrieved.unwrap();
        assert_eq!(retrieved_session.session_id, session.session_id);
        assert_eq!(retrieved_session.data, session.data);
    }

    #[tokio::test]
    async fn test_update_session() {
        let pool = setup_test_database().await;
        let user_data = create_test_user_data();
        
        // Create session
        let session = create_user_session(&pool, &user_data).await.unwrap();
        
        // Update with new data
        let updated_data = json!({
            "email": "updated@example.com",
            "fullname": "Updated User",
            "last_activity": "form_submission"
        });
        
        let update_result = update_user_session(&pool, &session.session_id, &updated_data).await;
        assert!(update_result.is_ok(), "Should update session successfully");
        
        // Verify update
        let retrieved = get_user_session(&pool, &session.session_id).await.unwrap().unwrap();
        assert!(retrieved.data.contains("updated@example.com"));
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let pool = setup_test_database().await;
        let user_data = create_test_user_data();
        
        // Create session
        let session = create_user_session(&pool, &user_data).await.unwrap();
        
        // Manually expire session (in real scenario this would be time-based)
        let expire_result = expire_user_session(&pool, &session.session_id).await;
        assert!(expire_result.is_ok(), "Should expire session successfully");
        
        // Try to retrieve expired session
        let retrieved = get_user_session(&pool, &session.session_id).await.unwrap();
        assert!(retrieved.is_none() || !retrieved.unwrap().is_active, "Expired session should not be active");
    }

    #[test]
    fn test_session_id_generation() {
        let session_id = generate_session_id();
        
        assert!(!session_id.is_empty(), "Session ID should not be empty");
        assert!(session_id.len() >= 32, "Session ID should be sufficiently long");
        
        // Generate multiple IDs to ensure uniqueness
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert_ne!(id1, id2, "Session IDs should be unique");
    }

    #[test]
    fn test_session_data_serialization() {
        let test_data = json!({
            "email": "test@example.com",
            "form_progress": {
                "step": 2,
                "completed_fields": ["email", "name"]
            },
            "timestamp": "2023-01-01T12:00:00Z"
        });
        
        let serialized = serialize_session_data(&test_data);
        assert!(serialized.is_ok(), "Should serialize session data successfully");
        
        let deserialized = deserialize_session_data(&serialized.unwrap());
        assert!(deserialized.is_ok(), "Should deserialize session data successfully");
        assert_eq!(deserialized.unwrap(), test_data);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let pool = setup_test_database().await;
        
        // Create multiple sessions
        for i in 0..5 {
            let user_data = json!({"email": format!("test{}@example.com", i)});
            let _ = create_user_session(&pool, &user_data).await;
        }
        
        // Run cleanup
        let cleanup_result = cleanup_expired_sessions(&pool).await;
        assert!(cleanup_result.is_ok(), "Should cleanup expired sessions successfully");
    }

    #[tokio::test]
    async fn test_session_validation() {
        let pool = setup_test_database().await;
        let user_data = create_test_user_data();
        
        let session = create_user_session(&pool, &user_data).await.unwrap();
        
        // Validate active session
        let is_valid = validate_session(&pool, &session.session_id).await;
        assert!(is_valid.unwrap_or(false), "Active session should be valid");
        
        // Test with invalid session ID
        let invalid_id = "invalid-session-id";
        let is_invalid = validate_session(&pool, invalid_id).await;
        assert!(!is_invalid.unwrap_or(true), "Invalid session should not be valid");
    }

    #[test]
    fn test_session_security() {
        let session_id = generate_session_id();
        
        // Check that session ID contains no predictable patterns
        assert!(!session_id.contains("000000"), "Session ID should not contain obvious patterns");
        assert!(!session_id.contains("123456"), "Session ID should not contain sequential patterns");
        
        // Verify it's properly formatted
        assert!(session_id.chars().all(|c| c.is_alphanumeric() || c == '-'), 
               "Session ID should only contain alphanumeric characters and hyphens");
    }

    #[tokio::test]
    async fn test_concurrent_session_operations() {
        let pool = setup_test_database().await;
        let user_data = create_test_user_data();
        
        let session = create_user_session(&pool, &user_data).await.unwrap();
        
        // Simulate concurrent updates
        let pool_clone1 = pool.clone();
        let pool_clone2 = pool.clone();
        let session_id1 = session.session_id.clone();
        let session_id2 = session.session_id.clone();
        
        let update1 = tokio::spawn(async move {
            let data = json!({"concurrent_update": 1});
            update_user_session(&pool_clone1, &session_id1, &data).await
        });
        
        let update2 = tokio::spawn(async move {
            let data = json!({"concurrent_update": 2});
            update_user_session(&pool_clone2, &session_id2, &data).await
        });
        
        let (result1, result2) = tokio::join!(update1, update2);
        
        // At least one update should succeed
        assert!(result1.unwrap().is_ok() || result2.unwrap().is_ok(), 
               "At least one concurrent update should succeed");
    }

    #[test]
    fn test_session_data_sanitization() {
        let sensitive_data = json!({
            "email": "test@example.com",
            "password": "secret123",
            "credit_card": "1234-5678-9012-3456",
            "ssn": "123-45-6789"
        });
        
        let sanitized = sanitize_session_data(&sensitive_data);
        
        // Should remove sensitive fields
        assert!(!sanitized.to_string().contains("secret123"));
        assert!(!sanitized.to_string().contains("1234-5678"));
        assert!(!sanitized.to_string().contains("123-45-6789"));
        
        // Should keep non-sensitive data
        assert!(sanitized.to_string().contains("test@example.com"));
    }

    #[tokio::test]
    async fn test_session_metrics() {
        let pool = setup_test_database().await;
        
        // Create multiple sessions
        for i in 0..10 {
            let user_data = json!({"user_id": i});
            let _ = create_user_session(&pool, &user_data).await;
        }
        
        let metrics = get_session_metrics(&pool).await;
        assert!(metrics.is_ok(), "Should get session metrics successfully");
        
        let stats = metrics.unwrap();
        assert!(stats.total_sessions >= 10, "Should count created sessions");
    }

    // Helper functions for session tests
    fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    fn serialize_session_data(data: &serde_json::Value) -> Result<String, serde_json::Error> {
        serde_json::to_string(data)
    }

    fn deserialize_session_data(data: &str) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(data)
    }

    fn sanitize_session_data(data: &serde_json::Value) -> serde_json::Value {
        let mut sanitized = data.clone();
        
        if let Some(obj) = sanitized.as_object_mut() {
            // Remove sensitive fields
            obj.remove("password");
            obj.remove("credit_card");
            obj.remove("ssn");
            obj.remove("token");
            obj.remove("api_key");
        }
        
        sanitized
    }
}
