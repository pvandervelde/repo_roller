//! Integration testing library for RepoRoller.
//!
//! This library provides utilities for integration testing of RepoRoller
//! functionality, including repository creation, template processing,
//! variable substitution, and error handling scenarios.

pub mod fixtures;
pub mod helpers;
pub mod test_runner;
pub mod utils;
pub mod verification;

// Re-export commonly used types for convenience
pub use test_runner::{IntegrationTestRunner, TestResult, TestScenario};
pub use utils::{generate_test_repo_name, RepositoryCleanup, TestConfig, TestRepository};
pub use verification::{
    ConfigurationVerification, ExpectedBranchProtection, ExpectedConfiguration,
    ExpectedRepositorySettings,
};
