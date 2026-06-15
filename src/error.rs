use thiserror::Error;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("path not found")]
    NotFound,
    #[error("access denied to path")]
    AccessDenied,
    #[error("something went wrong: {0}")]
    Fatal(String),
}
