/// The common types used in the Swiftlink API.
use serde::{Deserialize, Serialize};

/// Represents a request to create a new short link.
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateLinkRequest {
    /// The original URL to be shortened.
    pub url: String,
}

/// Represents the response containing the details of a newly created short link.
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateLinkResponse {
    /// The generated short code for the link.
    pub code: String,
    /// The shortened URL.
    pub url: String,
}

/// Represents the response containing information about an existing short link.
#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponse {
    /// The short code of the link.
    pub code: String,
    /// The original URL that the short link redirects to.
    pub url: String,
    /// The Unix timestamp (in seconds) when the short link was created.
    pub created_at: i64,
}
