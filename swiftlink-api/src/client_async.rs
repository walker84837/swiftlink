use crate::request_types::*;
use crate::{SwiftlinkClientError, SwiftlinkResult};
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct SwiftlinkClient {
    client: Client,
    base_url: String,
}

impl SwiftlinkClient {
    /// Creates a new server client with the given base URL (e.g., "http://localhost:8080") to use
    /// the API.
    pub fn new(base_url: impl Into<String>) -> Self {
        SwiftlinkClient {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Calls the `/api/create` endpoint to create a short link.
    pub async fn create_link(&self, url: impl AsRef<str>) -> SwiftlinkResult<CreateLinkResponse> {
        let final_url = url.as_ref();
        let req_body = CreateLinkRequest {
            url: final_url.to_string(),
        };
        let resp = self
            .client
            .post(format!("{}/api/create", self.base_url.as_str()))
            .json(&req_body)
            .send()
            .await
            .map_err(SwiftlinkClientError::RequestError)?
            .error_for_status()
            .map_err(SwiftlinkClientError::RequestError)?
            .json::<CreateLinkResponse>()
            .await
            .map_err(SwiftlinkClientError::RequestError)?;
        Ok(resp)
    }

    /// Calls the `/api/info/{code}` endpoint to retrieve link info.
    pub async fn get_link_info(&self, code: impl AsRef<str>) -> SwiftlinkResult<InfoResponse> {
        let resp = self
            .client
            .get(format!("{}/api/info/{}", self.base_url, code.as_ref()))
            .send()
            .await
            .map_err(SwiftlinkClientError::RequestError)?
            .error_for_status()
            .map_err(SwiftlinkClientError::RequestError)?
            .json::<InfoResponse>()
            .await
            .map_err(SwiftlinkClientError::RequestError)?;
        Ok(resp)
    }

    /// Calls the `/{code}` endpoint to get the redirection URL.
    ///
    /// The server should return a "Location" header on redirection.
    /// Returns an error when the header is not found.
    pub async fn redirect(&self, code: impl AsRef<str>) -> SwiftlinkResult<String> {
        let resp = self
            .client
            .get(format!("{}/{}", self.base_url, code.as_ref()))
            .send()
            .await
            .map_err(SwiftlinkClientError::RequestError)?
            .error_for_status()
            .map_err(SwiftlinkClientError::RequestError)?;

        if let Some(loc) = resp.headers().get("Location") {
            Ok(loc
                .to_str()
                .map_err(|e| SwiftlinkClientError::UnexpectedResponse(e.to_string()))?
                .to_string())
        } else {
            Err(SwiftlinkClientError::UnexpectedResponse(
                "Redirect location header not found".to_string(),
            ))
        }
    }
}
