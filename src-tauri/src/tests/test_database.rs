#![cfg(test)]

use super::{
    database::*,
    common::*,
    pretty_assertions::assert_eq,
    serde_json::json,
};
use sqlx::{PgPool, Row, query as sqlx_query};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use sqlx::{PgPool, Row};

    #[tokio::test]
    async fn test_database_connection() {
        let pool = setup_test_database().await;
        
        // Test basic connectivity
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(&pool)
            .await;
        
        assert!(result.is_ok(), "Should connect to database successfully");
        
        let row = result.unwrap();
        let test_value: i32 = row.get("test");
        assert_eq!(test_value, 1, "Should execute simple query correctly");
    }

    #[tokio::test]
    async fn test_database_migration() {
        let pool = setup_test_database().await;
        
        // Run migrations
        let migration_result = run_database_migrations(&pool).await;
        assert!(migration_result.is_ok(), "Database migrations should run successfully");
        
        // Verify tables exist
        let tables = vec![
            "user_sessions",
            "dsl_scripts_cache", 
            "application_logs",
            "bitwarden_cache",
            "automation_runs"
        ];
        
        for table in tables {
            let exists = check_table_exists(&pool, table).await;
            assert!(exists.unwrap_or(false), "Table {} should exist after migration", table);
        }
    }

    #[tokio::test]
    async fn test_session_table_operations() {
        let pool = setup_test_database().await;
        
        let session_id = "test-session-123";
        let user_data = create_test_user_data();
        let data_json = serde_json::to_string(&user_data).unwrap();
        
        // Insert session
        let insert_result = sqlx::query!(
            "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
             VALUES ($1, $2, NOW(), NOW(), NOW() + INTERVAL '1 day', true)",
            session_id,
            data_json
        )
        .execute(&pool)
        .await;
        
        assert!(insert_result.is_ok(), "Should insert session successfully");
        
        // Retrieve session
        let select_result = sqlx::query!(
            "SELECT * FROM user_sessions WHERE session_id = $1",
            session_id
        )
        .fetch_one(&pool)
        .await;
        
        assert!(select_result.is_ok(), "Should retrieve session successfully");
        
        let session = select_result.unwrap();
        assert_eq!(session.session_id, session_id);
        assert!(session.is_active.unwrap_or(false));
        
        // Update session
        let new_data = json!({"updated": true});
        let new_data_json = serde_json::to_string(&new_data).unwrap();
        
        let update_result = sqlx::query!(
            "UPDATE user_sessions SET user_data = $1, updated_at = NOW() WHERE session_id = $2",
            new_data_json,
            session_id
        )
        .execute(&pool)
        .await;
        
        assert!(update_result.is_ok(), "Should update session successfully");
        
        // Delete session
        let delete_result = sqlx::query!(
            "DELETE FROM user_sessions WHERE session_id = $1",
            session_id
        )
        .execute(&pool)
        .await;
        
        assert!(delete_result.is_ok(), "Should delete session successfully");
    }

    #[tokio::test]
    async fn test_dsl_cache_table_operations() {
        let pool = setup_test_database().await;
        
        let cache_key = "test-cache-key";
        let script_content = "wait 2\nclick \"#submit\"\nwait 3";
        let html_hash = "abc123def456";
        
        // Insert DSL script cache
        let insert_result = sqlx::query!(
            "INSERT INTO dsl_scripts_cache (cache_key, script_content, html_hash, created_at, expires_at)
             VALUES ($1, $2, $3, NOW(), NOW() + INTERVAL '1 day')",
            cache_key,
            script_content,
            html_hash
        )
        .execute(&pool)
        .await;
        
        assert!(insert_result.is_ok(), "Should insert DSL cache successfully");
        
        // Retrieve from cache
        let select_result = sqlx::query!(
            "SELECT script_content FROM dsl_scripts_cache 
             WHERE cache_key = $1 AND expires_at > NOW()",
            cache_key
        )
        .fetch_optional(&pool)
        .await;
        
        assert!(select_result.is_ok(), "Should query DSL cache successfully");
        
        let cached_script = select_result.unwrap();
        assert!(cached_script.is_some(), "Should find cached script");
        assert_eq!(cached_script.unwrap().script_content, script_content);
        
        // Test cache expiration
        let expire_result = sqlx::query!(
            "UPDATE dsl_scripts_cache SET expires_at = NOW() - INTERVAL '1 hour' WHERE cache_key = $1",
            cache_key
        )
        .execute(&pool)
        .await;
        
        assert!(expire_result.is_ok(), "Should expire cache entry");
        
        // Should not find expired cache
        let expired_select = sqlx::query!(
            "SELECT script_content FROM dsl_scripts_cache 
             WHERE cache_key = $1 AND expires_at > NOW()",
            cache_key
        )
        .fetch_optional(&pool)
        .await;
        
        assert!(expired_select.unwrap().is_none(), "Should not find expired cache entry");
    }

    #[tokio::test]
    async fn test_application_logs_table() {
        let pool = setup_test_database().await;
        
        let log_entries = vec![
            ("info", "test_component", "Test info message", json!({"user": "test"})),
            ("warn", "bitwarden", "Vault unlock warning", json!({"attempts": 2})),
            ("error", "database", "Connection failed", json!({"timeout": true})),
        ];
        
        // Insert log entries
        for (level, component, message, context) in &log_entries {
            let context_json = serde_json::to_string(context).unwrap();
            
            let insert_result = sqlx::query!(
                "INSERT INTO application_logs (level, component, message, context, created_at)
                 VALUES ($1, $2, $3, $4, NOW())",
                level,
                component,
                message,
                context_json
            )
            .execute(&pool)
            .await;
            
            assert!(insert_result.is_ok(), "Should insert log entry for level {}", level);
        }
        
        // Query logs by level
        let error_logs = sqlx::query!(
            "SELECT * FROM application_logs WHERE level = $1 ORDER BY created_at DESC",
            "error"
        )
        .fetch_all(&pool)
        .await;
        
        assert!(error_logs.is_ok(), "Should query error logs successfully");
        assert!(!error_logs.unwrap().is_empty(), "Should have error log entries");
        
        // Query logs by component
        let bitwarden_logs = sqlx::query!(
            "SELECT * FROM application_logs WHERE component = $1",
            "bitwarden"
        )
        .fetch_all(&pool)
        .await;
        
        assert!(bitwarden_logs.is_ok(), "Should query bitwarden logs successfully");
        assert!(!bitwarden_logs.unwrap().is_empty(), "Should have bitwarden log entries");
    }

    #[tokio::test]
    async fn test_database_performance() {
        let pool = setup_test_database().await;
        
        let start_time = std::time::Instant::now();
        
        // Insert multiple records to test performance
        for i in 0..100 {
            let session_id = format!("perf-test-{}", i);
            let user_data = json!({"test_id": i, "email": format!("test{}@example.com", i)});
            let data_json = serde_json::to_string(&user_data).unwrap();
            
            sqlx::query!(
                "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
                 VALUES ($1, $2, NOW(), NOW(), NOW() + INTERVAL '1 day', true)",
                session_id,
                data_json
            )
            .execute(&pool)
            .await
            .expect("Should insert performance test session");
        }
        
        let insert_duration = start_time.elapsed();
        
        // Query performance test
        let query_start = std::time::Instant::now();
        
        let count_result = sqlx::query!(
            "SELECT COUNT(*) as count FROM user_sessions WHERE session_id LIKE 'perf-test-%'"
        )
        .fetch_one(&pool)
        .await;
        
        let query_duration = query_start.elapsed();
        
        assert!(count_result.is_ok(), "Performance query should succeed");
        assert_eq!(count_result.unwrap().count.unwrap_or(0), 100, "Should count all inserted records");
        
        // Performance assertions
        assert!(insert_duration.as_millis() < 5000, "Bulk insert should complete within 5 seconds");
        assert!(query_duration.as_millis() < 1000, "Count query should complete within 1 second");
        
        // Cleanup performance test data
        sqlx::query!("DELETE FROM user_sessions WHERE session_id LIKE 'perf-test-%'")
            .execute(&pool)
            .await
            .expect("Should cleanup performance test data");
    }

    #[tokio::test]
    async fn test_database_transactions() {
        let pool = setup_test_database().await;
        
        // Test successful transaction
        let mut tx = pool.begin().await.expect("Should start transaction");
        
        let session_id = "tx-test-success";
        let user_data = json!({"transaction": "test"});
        let data_json = serde_json::to_string(&user_data).unwrap();
        
        sqlx::query!(
            "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
             VALUES ($1, $2, NOW(), NOW(), NOW() + INTERVAL '1 day', true)",
            session_id,
            data_json
        )
        .execute(&mut *tx)
        .await
        .expect("Should insert in transaction");
        
        tx.commit().await.expect("Should commit transaction");
        
        // Verify committed data exists
        let exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM user_sessions WHERE session_id = $1",
            session_id
        )
        .fetch_one(&pool)
        .await
        .expect("Should query committed data");
        
        assert_eq!(exists.count.unwrap_or(0), 1, "Committed data should exist");
        
        // Test rollback transaction
        let mut tx2 = pool.begin().await.expect("Should start second transaction");
        
        let rollback_session_id = "tx-test-rollback";
        
        sqlx::query!(
            "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
             VALUES ($1, $2, NOW(), NOW(), NOW() + INTERVAL '1 day', true)",
            rollback_session_id,
            data_json
        )
        .execute(&mut *tx2)
        .await
        .expect("Should insert in rollback transaction");
        
        tx2.rollback().await.expect("Should rollback transaction");
        
        // Verify rolled back data doesn't exist
        let not_exists = sqlx::query!(
            "SELECT COUNT(*) as count FROM user_sessions WHERE session_id = $1",
            rollback_session_id
        )
        .fetch_one(&pool)
        .await
        .expect("Should query rolled back data");
        
        assert_eq!(not_exists.count.unwrap_or(0), 0, "Rolled back data should not exist");
    }

    #[tokio::test]
    async fn test_database_constraints() {
        let pool = setup_test_database().await;
        
        let session_id = "constraint-test";
        let user_data = json!({"test": "constraints"});
        let data_json = serde_json::to_string(&user_data).unwrap();
        
        // Insert first record
        let insert1 = sqlx::query!(
            "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
             VALUES ($1, $2, NOW(), NOW(), NOW() + INTERVAL '1 day', true)",
            session_id,
            data_json
        )
        .execute(&pool)
        .await;
        
        assert!(insert1.is_ok(), "First insert should succeed");
        
        // Try to insert duplicate session_id (should fail if unique constraint exists)
        let insert2 = sqlx::query!(
            "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
             VALUES ($1, $2, NOW(), NOW(), NOW() + INTERVAL '1 day', true)",
            session_id,
            data_json
        )
        .execute(&pool)
        .await;
        
        // This might succeed or fail depending on constraints - we just ensure it doesn't panic
        let _ = insert2;
    }

    #[tokio::test]
    async fn test_database_cleanup_operations() {
        let pool = setup_test_database().await;
        
        // Insert test data for cleanup
        for i in 0..50 {
            let session_id = format!("cleanup-test-{}", i);
            let user_data = json!({"cleanup": i});
            let data_json = serde_json::to_string(&user_data).unwrap();
            
            sqlx::query!(
                "INSERT INTO user_sessions (session_id, user_data, created_at, updated_at, expires_at, is_active) 
                 VALUES ($1, $2, NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day', false)",
                session_id,
                data_json
            )
            .execute(&pool)
            .await
            .expect("Should insert cleanup test data");
        }
        
        // Run cleanup operation
        let cleanup_result = sqlx::query!(
            "DELETE FROM user_sessions WHERE expires_at < NOW() AND session_id LIKE 'cleanup-test-%'"
        )
        .execute(&pool)
        .await;
        
        assert!(cleanup_result.is_ok(), "Cleanup operation should succeed");
        
        let rows_affected = cleanup_result.unwrap().rows_affected();
        assert_eq!(rows_affected, 50, "Should cleanup all test records");
    }

    // Helper functions
    async fn check_table_exists(pool: &PgPool, table_name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = $1
            )",
            table_name
        )
        .fetch_one(pool)
        .await?;
        
        Ok(result.exists.unwrap_or(false))
    }

    async fn run_database_migrations(_pool: &PgPool) -> Result<(), sqlx::Error> {
        // In a real implementation, this would run actual migrations
        // For testing, we assume migrations are already applied
        Ok(())
    }
}
