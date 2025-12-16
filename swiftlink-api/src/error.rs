use thiserror::Error;

/// A specialized `Result` type for Swiftlink client operations.
///
/// This type is used as the return type for fallible Swiftlink client functions,
/// returning [`enum@SwiftlinkClientError`] on failure.
pub type SwiftlinkResult<T> = Result<T, crate::error::SwiftlinkClientError>;

/// Represents the possible errors that can occur when using the Swiftlink client.
#[derive(Error, Debug)]
pub enum SwiftlinkClientError {
    /// An error occurred during an HTTP request.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    /// An I/O error occurred.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// The server returned an unexpected or malformed response.
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}
