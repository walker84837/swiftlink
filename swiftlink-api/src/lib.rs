/// Async client implementation
#[cfg(feature = "async")]
pub mod client_async;

/// Blocking client implementation
#[cfg(feature = "blocking")]
pub mod client_blocking;
pub mod error;
pub mod request_types;

// Re-export the client and common types so that users of the library have a unified API.
#[cfg(feature = "async")]
pub use client_async::SwiftlinkClient as AsyncSwiftlinkClient;
#[cfg(feature = "blocking")]
pub use client_blocking::SwiftlinkClient as BlockingSwiftlinkClient;

pub use error::{SwiftlinkClientError, SwiftlinkResult};
pub use request_types::CreateLinkRequest;
pub use request_types::CreateLinkResponse;
pub use request_types::InfoResponse;
