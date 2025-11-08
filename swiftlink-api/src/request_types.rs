/// The common types used in the Swiftlink API.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateLinkRequest {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateLinkResponse {
    pub code: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponse {
    pub code: String,
    pub url: String,
    pub created_at: i64,
}
