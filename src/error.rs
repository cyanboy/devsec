use sqlx::migrate::MigrateError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Api error: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("CSV Error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Migration Error {0}")]
    MigrationError(#[from] MigrateError),
}

impl AppError {
    pub fn api_error(status_code: u16, message: impl Into<String>) -> Self {
        Self::ApiError {
            status_code,
            message: message.into(),
        }
    }
}
