use thiserror::Error;

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

#[derive(Error, Debug)]
pub enum Error {
    #[error("An unknown error occurred.")]
    Unknown,
}
