//! Test modules for the codialog application

// Always-available shared helpers for tests
pub mod common;

// Compile specific test groups only when requested via Cargo features
#[cfg(feature = "tests_llm")]
pub mod test_llm;

#[cfg(feature = "tests_logging")]
pub mod test_logging;

#[cfg(feature = "tests_session")]
pub mod test_session;

#[cfg(feature = "tests_database")]
pub mod test_database;

#[cfg(feature = "integration_tests")]
pub mod integration_tests;

// Optional Bitwarden tests (disabled by default)
#[cfg(feature = "tests_bitwarden")]
pub mod test_bitwarden;

// Common test utilities (kept minimal to avoid import ambiguity)
pub mod utils {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use serde_json::json;

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
