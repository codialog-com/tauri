//! Test modules for the codialog application

// Test modules
pub mod common;
pub mod test_bitwarden;
pub mod test_database;
pub mod test_llm;
pub mod test_logging;
pub mod test_session;
pub mod integration_tests;

// Re-export commonly used test utilities
pub use crate::{
    bitwarden,
    llm,
    logging,
    session,
    tagui,
};

// Re-export test utilities
pub use common::{
    create_test_credentials,
};

// Re-export test dependencies
#[doc(hidden)]
pub use pretty_assertions;
#[doc(hidden)]
pub use serde_json;
#[doc(hidden)]
pub use sqlx;
#[doc(hidden)]
pub use tokio::test as tokio_test;

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
