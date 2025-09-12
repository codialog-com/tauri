//! Test modules for the codialog application

// Test modules
pub mod test_bitwarden;
pub mod test_llm;
pub mod test_session;
pub mod test_logging;
pub mod test_database;
pub mod integration_tests;

// Common test utilities
#[cfg(test)]
pub mod common;

// Re-export common test utilities for easier access in tests
#[cfg(test)]
pub use common::{
    create_test_html_form,
    create_complex_html_form,
    create_test_user_data,
    create_test_credentials,
    setup_test_database
};
