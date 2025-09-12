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
pub use common::*;

// Re-export main modules for testing
#[cfg(test)]
pub use crate::{
    bitwarden,
    llm,
    logging,
    session,
    database,
    tagui,
};

// Re-export commonly used test dependencies
#[cfg(test)]
pub use pretty_assertions::assert_eq;
#[cfg(test)]
pub use serde_json::json;
#[cfg(test)]
pub use sqlx::query;
#[cfg(test)]
pub use tokio::test as tokio_test;

// Common test utilities
#[cfg(test)]
pub use common::{
    create_test_html_form,
    create_complex_html_form,
    create_test_user_data,
    create_test_credentials,
    setup_test_database,
    TestDatabase,
};
