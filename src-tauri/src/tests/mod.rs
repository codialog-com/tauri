//! Test modules for the codialog application

// Test modules
pub mod common;
pub mod test_bitwarden;
pub mod test_database;
pub mod test_llm;
pub mod test_logging;
pub mod test_session;
pub mod integration_tests;

// Re-export test utilities
pub use common::{
    create_test_credentials,
    setup_test_database,
    create_test_user,
    create_test_session,
    create_test_log_entry,
};

// Re-export common test dependencies
pub use pretty_assertions::assert_eq;
pub use serde_json::json;
pub use tempfile::tempdir;
pub use tokio::test;
pub use serde_json;
pub use sqlx;

// Re-export commonly used modules and functions
pub use crate::{
    bitwarden::{
        check_bitwarden_status,
        parse_bitwarden_credentials,
        bitwarden_login,
    },
    database::{
        setup_test_database as db_setup,
        get_db_pool,
        run_migrations,
        execute_sql,
    },
    dsl::{
        execute_dsl_script,
    },
    llm::{
        LLMRequest,
        LLMResponse,
        generate_fallback_script,
        analyze_form_structure,
        process_natural_language_query,
        get_llm_response,
        FormAnalysis,
        FormField,
        FieldType,
    },
    logging::{
        initialize_logging_system,
        log_user_action,
        get_application_logs,
        get_logs_by_level,
        get_log_statistics,
        write_log_to_file,
        rotate_log_file,
        log_performance_metric,
        get_performance_logs,
        cleanup_old_logs,
    },
    session::{
        create_user_session,
        get_user_session,
        expire_user_session,
        cleanup_expired_sessions,
        validate_session,
        update_user_session,
        get_session_metrics,
    },
};

// Common test utilities module
pub mod utils {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;
    use chrono::{Utc, Duration};
    
    // Helper function to create a test database connection
    pub async fn setup_test_db() -> sqlx::PgPool {
        setup_test_database().await
    }
    
    // Helper function to create a test user
    pub fn create_test_user() -> serde_json::Value {
        json!({
            "user_id": Uuid::new_v4().to_string(),
            "email": format!("test_user_{}@example.com", Uuid::new_v4()),
            "name": "Test User",
            "created_at": Utc::now().to_rfc3339(),
        })
    }
}
