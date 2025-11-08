use thiserror::Error;

pub type SwiftlinkResult<T> = Result<T, SwiftlinkClientError>;

#[derive(Error, Debug)]
pub enum SwiftlinkClientError {
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}
