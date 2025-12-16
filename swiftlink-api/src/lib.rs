//! A client library for interacting with the Swiftlink API.
//!
//! This crate provides both `AsyncSwiftlinkClient` and `BlockingSwiftlinkClient` clients to interact
//! with the Swiftlink URL shortening service. It also exposes the request/response types such as
//! [`CreateLinkRequest`], [`CreateLinkResponse`] and [`InfoResponse`], and the error types in the
//! [`crate::error`] module (for example [`SwiftlinkClientError`]).

#![warn(missing_docs)]
#![forbid(unsafe_code)]

/// Async client implementation
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub mod client_async;

/// Blocking client implementation
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub mod client_blocking;

/// Error types for the Swiftlink client.
pub mod error;
/// Request and response types for the Swiftlink API.
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
