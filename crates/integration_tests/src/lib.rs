//! Integration testing library for RepoRoller.
//!
//! This library provides utilities for integration testing of RepoRoller
//! functionality, including repository creation, template processing,
//! variable substitution, and error handling scenarios.

pub mod fixtures;
pub mod helpers;
pub mod mock_github;
pub mod test_runner;
pub mod utils;
pub mod verification;

// Re-export commonly used types for convenience
pub use test_runner::{IntegrationTestRunner, TestResult, TestScenario};
pub use utils::{is_test_repository, RepositoryCleanup, TestConfig, TestRepository};
pub use verification::{
    ConfigurationVerification, ExpectedBranchProtection, ExpectedConfiguration,
    ExpectedRepositorySettings,
};

// Re-export test_utils functions
pub use test_utils::{cleanup_test_repository, generate_test_repo_name, get_workflow_context};
