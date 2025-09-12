#![cfg(test)]

use super::{
    bitwarden::*,
    llm::*,
    logging::*,
    session::*,
    database::*,
    common::*,
};
use pretty_assertions::assert_eq as pretty_assert_eq;
use serde_json::json;
use sqlx::query as sqlx_query;

mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_full_bitwarden_dsl_workflow() {
        let pool = setup_test_database().await;
        let html = create_test_html_form();
        let user_data = create_test_user_data();
        
        // Test complete workflow: Bitwarden + DSL generation + Session management
        
        // 1. Create user session
        let session = session::create_user_session(&pool, &user_data).await;
        assert!(session.is_ok(), "Should create user session");
        let session = session.unwrap();
        
        // 2. Generate DSL script with caching
        let dsl_script = llm::generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        assert!(!dsl_script.is_empty(), "Should generate DSL script");
        assert!(dsl_script.contains("type"), "DSL should contain type commands");
        
        // 3. Update session with automation data
        let automation_data = json!({
            "dsl_script": dsl_script,
            "target_url": "https://example.com/login",
            "automation_status": "ready"
        });
        
        let session_update = session::update_user_session(&pool, &session.session_id, &automation_data).await;
        assert!(session_update.is_ok(), "Should update session with automation data");
        
        // 4. Retrieve updated session
        let retrieved_session = session::get_user_session(&pool, &session.session_id).await;
        assert!(retrieved_session.is_ok(), "Should retrieve updated session");
        assert!(retrieved_session.unwrap().is_some(), "Session should exist");
    }

    #[tokio::test]
    async fn test_bitwarden_integration_workflow() {
        // Test Bitwarden CLI integration workflow
        
        // 1. Check Bitwarden status
        let status = bitwarden::check_bitwarden_status().await;
        assert!(!status.is_empty(), "Should return Bitwarden status");
        
        // 2. Test credential parsing (with mock data)
        let mock_credentials_json = r#"[
            {
                "id": "mock-id-1",
                "name": "Test Site 1",
                "login": {
                    "username": "user1@example.com",
                    "password": "password123"
                },
                "uris": [{"uri": "https://example.com"}]
            }
        ]"#;
        
        let credentials = bitwarden::parse_bitwarden_credentials(mock_credentials_json);
        assert_eq!(credentials.len(), 1, "Should parse credentials correctly");
        assert_eq!(credentials[0].username.as_ref().unwrap(), "user1@example.com");
        
        // 3. Test credential filtering
        let filtered: Vec<_> = credentials.into_iter()
            .filter(|cred| cred.uri.as_ref()
                .map(|uri| uri.contains("example.com"))
                .unwrap_or(false))
            .collect();
        
        assert_eq!(filtered.len(), 1, "Should filter credentials by domain");
    }

    #[tokio::test]
    async fn test_logging_integration() {
        let pool = setup_test_database().await;
        
        // Test integrated logging across components
        
        // 1. Log Bitwarden operation
        logging::log_system_event(&pool, "bitwarden", "info", &json!({
            "operation": "vault_unlock",
            "status": "success",
            "items_count": 15
        })).await.expect("Should log Bitwarden event");
        
        // 2. Log DSL generation
        logging::log_system_event(&pool, "dsl_generator", "info", &json!({
            "operation": "script_generation", 
            "method": "enhanced",
            "script_length": 150,
            "cached": false
        })).await.expect("Should log DSL generation event");
        
        // 3. Log user action
        logging::log_user_action(&pool, "test_user", "form_submission", &json!({
            "form_type": "job_application",
            "fields_filled": 8,
            "success": true
        })).await.expect("Should log user action");
        
        // 4. Retrieve and verify logs
        let logs = logging::get_application_logs(&pool, Some(10), None).await;
        assert!(logs.is_ok(), "Should retrieve application logs");
        
        let log_entries = logs.unwrap();
        assert!(log_entries.len() >= 3, "Should have logged all events");
        
        // 5. Test log filtering
        let bitwarden_logs = logging::get_logs_by_component(&pool, "bitwarden", Some(5)).await;
        assert!(bitwarden_logs.is_ok(), "Should filter logs by component");
        
        let bw_logs = bitwarden_logs.unwrap();
        assert!(!bw_logs.is_empty(), "Should have Bitwarden logs");
    }

    #[tokio::test]
    async fn test_caching_integration() {
        let pool = setup_test_database().await;
        let html = create_complex_html_form();
        let user_data = create_test_user_data();
        
        // Test DSL script caching integration
        
        // 1. Generate script (should cache)
        let script1 = llm::generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        assert!(!script1.is_empty(), "Should generate first script");
        
        // 2. Generate same script again (should use cache)
        let start_time = std::time::Instant::now();
        let script2 = llm::generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        let cache_duration = start_time.elapsed();
        
        assert_eq!(script1, script2, "Cached script should match original");
        assert!(cache_duration < Duration::from_millis(100), "Cache retrieval should be fast");
        
        // 3. Test cache invalidation
        let modified_html = html.replace("input", "INPUT"); // Change case
        let script3 = llm::generate_dsl_script_with_cache(&modified_html, &user_data, Some(&pool)).await;
        
        // Should generate new script for modified HTML
        assert!(!script3.is_empty(), "Should generate script for modified HTML");
    }

    #[tokio::test]
    async fn test_error_handling_integration() {
        let pool = setup_test_database().await;
        
        // Test error handling across components
        
        // 1. Test DSL generation with invalid input
        let invalid_html = "";
        let invalid_user_data = json!("not_an_object");
        
        let result = llm::generate_dsl_script_with_cache(&invalid_html, &invalid_user_data, Some(&pool)).await;
        assert!(!result.is_empty(), "Should handle invalid input gracefully");
        
        // 2. Test session operations with invalid data
        let invalid_session_result = session::get_user_session(&pool, "non-existent-session").await;
        assert!(invalid_session_result.is_ok(), "Should handle non-existent session gracefully");
        assert!(invalid_session_result.unwrap().is_none(), "Should return None for non-existent session");
        
        // 3. Test Bitwarden operations error handling
        let invalid_credentials_result = bitwarden::bitwarden_login("", "").await;
        assert!(invalid_credentials_result.is_err(), "Should handle invalid credentials appropriately");
    }

    #[tokio::test] 
    async fn test_performance_integration() {
        let pool = setup_test_database().await;
        
        // Test performance across integrated components
        
        let start_time = std::time::Instant::now();
        
        // 1. Create multiple sessions concurrently
        let mut session_handles = vec![];
        for i in 0..10 {
            let pool_clone = pool.clone();
            let user_data = json!({"user_id": i, "email": format!("perf{}@example.com", i)});
            
            let handle = tokio::spawn(async move {
                session::create_user_session(&pool_clone, &user_data).await
            });
            session_handles.push(handle);
        }
        
        // 2. Generate multiple DSL scripts concurrently
        let mut dsl_handles = vec![];
        for i in 0..5 {
            let pool_clone = pool.clone();
            let html = format!("{}<input id='field{}'>", create_test_html_form(), i);
            let user_data = json!({"test_id": i});
            
            let handle = tokio::spawn(async move {
                llm::generate_dsl_script_with_cache(&html, &user_data, Some(&pool_clone)).await
            });
            dsl_handles.push(handle);
        }
        
        // Wait for all operations to complete
        let session_results = futures::future::join_all(session_handles).await;
        let dsl_results = futures::future::join_all(dsl_handles).await;
        
        let total_duration = start_time.elapsed();
        
        // Verify all operations succeeded
        for result in session_results {
            assert!(result.unwrap().is_ok(), "Concurrent session creation should succeed");
        }
        
        for result in dsl_results {
            let script = result.unwrap();
            assert!(!script.is_empty(), "Concurrent DSL generation should succeed");
        }
        
        // Performance assertion
        assert!(total_duration < Duration::from_secs(10), "Concurrent operations should complete within 10 seconds");
        
        // 3. Log performance metrics
        logging::log_performance_metric(&pool, "integration_test", total_duration.as_millis() as i64, &json!({
            "sessions_created": 10,
            "dsl_scripts_generated": 5,
            "concurrent": true
        })).await.expect("Should log performance metrics");
    }

    #[tokio::test]
    async fn test_data_consistency_integration() {
        let pool = setup_test_database().await;
        
        // Test data consistency across operations
        
        let user_data = create_test_user_data();
        let html = create_test_html_form();
        
        // 1. Create session
        let session = session::create_user_session(&pool, &user_data).await.unwrap();
        
        // 2. Generate and cache DSL script
        let dsl_script = llm::generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        
        // 3. Update session with DSL data
        let session_data = json!({
            "original_data": user_data,
            "generated_dsl": dsl_script,
            "workflow_stage": "dsl_generated"
        });
        
        session::update_user_session(&pool, &session.session_id, &session_data).await.unwrap();
        
        // 4. Verify data consistency
        let retrieved_session = session::get_user_session(&pool, &session.session_id).await.unwrap().unwrap();
        
        // Parse and verify session data
        let parsed_data: serde_json::Value = serde_json::from_str(&retrieved_session.data).unwrap();
        assert!(parsed_data.get("generated_dsl").is_some(), "Session should contain DSL script");
        assert!(parsed_data.get("original_data").is_some(), "Session should contain original user data");
        assert_eq!(
            parsed_data.get("workflow_stage").unwrap().as_str().unwrap(), 
            "dsl_generated",
            "Workflow stage should be updated"
        );
        
        // 5. Verify DSL script is cached
        let cached_script = llm::generate_dsl_script_with_cache(&html, &user_data, Some(&pool)).await;
        assert_eq!(cached_script, dsl_script, "DSL script should be retrieved from cache");
    }

    #[tokio::test]
    async fn test_cleanup_integration() {
        let pool = setup_test_database().await;
        
        // Test cleanup operations across all components
        
        // Create test data
        for i in 0..20 {
            // Create expired sessions
            let user_data = json!({"cleanup_test": i});
            let session = session::create_user_session(&pool, &user_data).await.unwrap();
            
            // Manually expire session
            sqlx::query(
                "UPDATE user_sessions SET expires_at = NOW() - INTERVAL '1 day' WHERE session_id = $1"
            )
            .bind(&session.session_id)
            .execute(&pool)
            .await
            .expect("Should expire test session");
            
            // Create expired DSL cache entries
            sqlx::query(
                "INSERT INTO dsl_scripts_cache (cache_key, script_content, html_hash, created_at, expires_at)
                 VALUES ($1, $2, $3, NOW(), NOW() - INTERVAL '1 day')"
            )
            .bind(format!("cleanup_test_{}", i))
            .bind("test script content")
            .bind("test_hash")
            .execute(&pool)
            .await
            .expect("Should insert expired cache entry");
            
            // Create old log entries
            logging::log_system_event(&pool, "cleanup_test", "info", &json!({"test_id": i})).await.unwrap();
        }
        
        // Manually set old timestamps for logs
        sqlx::query(
            "UPDATE application_logs SET created_at = NOW() - INTERVAL '35 days' WHERE component = 'cleanup_test'"
        )
        .execute(&pool)
        .await
        .expect("Should age test log entries");
        
        // Run cleanup operations
        let session_cleanup = session::cleanup_expired_sessions(&pool).await;
        assert!(session_cleanup.is_ok(), "Should cleanup expired sessions");
        
        let cache_cleanup = sqlx::query("DELETE FROM dsl_scripts_cache WHERE expires_at < NOW()")
            .execute(&pool).await;
        assert!(cache_cleanup.is_ok(), "Should cleanup expired cache entries");
        
        let log_cleanup = logging::cleanup_old_logs(&pool, 30).await;
        assert!(log_cleanup.is_ok(), "Should cleanup old logs");
        
        // Verify cleanup results
        let remaining_sessions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM user_sessions WHERE expires_at < NOW()"
        )
        .fetch_one(&pool)
        .await
        .expect("Should query remaining sessions");
        
        assert_eq!(remaining_sessions.0, 0, "Should have no expired sessions remaining");
    }

    #[tokio::test]
    async fn test_security_integration() {
        let pool = setup_test_database().await;
        
        // Test security measures across components
        
        // 1. Test session security
        let user_data = json!({
            "email": "security@example.com",
            "password": "sensitive_password",
            "credit_card": "1234-5678-9012-3456"
        });
        
        let session = session::create_user_session(&pool, &user_data).await.unwrap();
        let retrieved = session::get_user_session(&pool, &session.session_id).await.unwrap().unwrap();
        
        // Verify sensitive data handling in session
        assert!(!retrieved.data.contains("sensitive_password"), "Password should not be stored in session");
        
        // 2. Test logging security (sensitive data sanitization)
        logging::log_user_action(&pool, "security_test", "login_attempt", &json!({
            "email": "security@example.com",
            "password": "should_be_redacted",
            "result": "success"
        })).await.expect("Should log with sensitive data");
        
        let security_logs = logging::get_logs_by_component(&pool, "security_test", Some(5)).await;
        // In a real implementation, verify that passwords are redacted in logs
        
        // 3. Test DSL script generation with sensitive data
        let sensitive_html = r#"<input type="password" name="secret">"#;
        let dsl_script = llm::generate_dsl_script_with_cache(&sensitive_html, &user_data, Some(&pool)).await;
        
        // DSL script should not contain raw sensitive data
        assert!(!dsl_script.contains("sensitive_password"), "DSL should not contain raw passwords");
    }
}
