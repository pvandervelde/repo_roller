//! Integration testing library for RepoRoller.
//!
//! This library provides utilities and test runners for comprehensive end-to-end
//! testing of RepoRoller functionality, including repository creation, template
//! processing, variable substitution, and error handling scenarios.

pub mod test_runner;
pub mod utils;

// Re-export commonly used types for convenience
pub use test_runner::{IntegrationTestRunner, TestResult, TestScenario};
pub use utils::{generate_test_repo_name, RepositoryCleanup, TestConfig, TestRepository};
