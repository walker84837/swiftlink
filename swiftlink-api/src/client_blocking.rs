use crate::{
    CreateLinkRequest, CreateLinkResponse, InfoResponse, SwiftlinkClientError, SwiftlinkResult,
};
use reqwest::blocking::Client;

#[derive(Debug, Clone)]
pub struct SwiftlinkClient {
    client: Client,
    base_url: String,
}

impl SwiftlinkClient {
    /// Creates a new SwiftlinkClient with the given base URL (e.g., "http://localhost:8080")
    pub fn new(base_url: impl Into<String>) -> Self {
        SwiftlinkClient {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Calls the `/api/create` endpoint to create a short link.
    pub fn create_link(&self, url: impl AsRef<str>) -> SwiftlinkResult<CreateLinkResponse> {
        let req_body = CreateLinkRequest {
            url: url.as_ref().to_string(),
        };
        let resp = self
            .client
            .post(format!("{}/api/create", self.base_url))
            .json(&req_body)
            .send()
            .map_err(SwiftlinkClientError::RequestError)?
            .error_for_status()
            .map_err(SwiftlinkClientError::RequestError)?
            .json::<CreateLinkResponse>()
            .map_err(SwiftlinkClientError::RequestError)?;
        Ok(resp)
    }

    /// Calls the `/api/info/{code}` endpoint to retrieve link info.
    pub fn get_link_info(&self, code: impl AsRef<str>) -> SwiftlinkResult<InfoResponse> {
        let resp = self
            .client
            .get(format!("{}/api/info/{}", self.base_url, code.as_ref()))
            .send()
            .map_err(SwiftlinkClientError::RequestError)?
            .error_for_status()
            .map_err(SwiftlinkClientError::RequestError)?
            .json::<InfoResponse>()
            .map_err(SwiftlinkClientError::RequestError)?;
        Ok(resp)
    }

    /// Calls the `/{code}` endpoint to get the redirection URL.
    ///
    /// Assumes that the server returns a "Location" header on redirection.
    pub fn redirect(&self, code: impl AsRef<str>) -> SwiftlinkResult<String> {
        let resp = self
            .client
            .get(format!("{}/{}", self.base_url, code.as_ref()))
            .send()
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

    /// Calls the `/{code}` endpoint to delete a short link.
    pub fn delete_link(
        &self,
        code: impl AsRef<str>,
        token: impl AsRef<str>,
    ) -> SwiftlinkResult<()> {
        self.client
            .delete(format!("{}/{}", self.base_url, code.as_ref()))
            .header("Authorization", format!("Bearer {}", token.as_ref()))
            .send()
            .map_err(SwiftlinkClientError::RequestError)?
            .error_for_status()
            .map_err(SwiftlinkClientError::RequestError)?;
        Ok(())
    }
}
