//! HTTP request and response models
//!
//! This module contains all HTTP-specific types for requests and responses.
//! These types are distinct from domain types and exist only in the HTTP layer.
//!
//! See specifications:
//! - `specs/interfaces/api-request-types.md`
//! - `specs/interfaces/api-response-types.md`

pub mod request;
pub mod response;

// Re-export commonly used types
pub use request::{
    CreateRepositoryRequest, PreviewConfigurationRequest, ValidateRepositoryNameRequest,
};
pub use response::{CreateRepositoryResponse, ListTemplatesResponse};
