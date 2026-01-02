use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("LINEAR_API_KEY environment variable not set")]
    MissingApiKey,

    #[error("Authentication failed: invalid API key")]
    Unauthorized,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::MissingApiKey | Error::Unauthorized => 2,
            Error::NotFound(_) => 3,
            Error::RateLimited(_) => 4,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
