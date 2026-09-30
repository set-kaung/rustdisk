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

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::AccessDenied,
            _ => Self::Fatal(value.to_string()),
        }
    }
}
