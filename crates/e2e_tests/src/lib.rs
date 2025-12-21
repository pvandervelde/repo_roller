//! E2E test utilities and support modules.

pub mod container;

pub use container::{ApiContainer, ApiContainerConfig};

// Re-export test utilities
pub use test_utils::cleanup_test_repository;
