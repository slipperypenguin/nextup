// custom error type using `anyhow` for simple error handling
pub type Result<T> = anyhow::Result<T>;

// Application-specific error types
#[derive(Debug)]
pub enum AppError {
    NamesFileError(std::io::Error),
    NoNamesFound,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NamesFileError(e) => write!(f, "Failed to read names file: {}", e),
            AppError::NoNamesFound => write!(f, "No names found in file"),
        }
    }
}

impl std::error::Error for AppError {}