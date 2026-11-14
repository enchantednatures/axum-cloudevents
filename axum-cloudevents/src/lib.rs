//! CloudEvents 1.0 extractors for [Axum](https://github.com/tokio-rs/axum).
//!
//! `axum-cloudevents` implements [`FromRequest`](axum::extract::FromRequest) for
//! [`CloudEvent<T>`], so a handler can accept a typed CloudEvent directly:
//!
//! ```
//! use axum::Json;
//! use axum_cloudevents::CloudEvent;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct Ping {
//!     message: String,
//! }
//!
//! #[derive(Serialize)]
//! struct Pong {
//!     reply: String,
//!     event_id: String,
//! }
//!
//! async fn handler(event: CloudEvent<Ping>) -> Json<Pong> {
//!     Json(Pong {
//!         reply: format!("pong: {}", event.data.message),
//!         event_id: event.id().to_string(),
//!     })
//! }
//! ```
//!
//! # Content modes
//!
//! Both CloudEvents content modes are supported:
//!
//! - **Structured**: a JSON body with the `application/cloudevents+json`
//!   envelope (`specversion`, `id`, `source`, `type`, ...).
//! - **Binary**: CloudEvents attributes in `ce-*` headers, with the payload in
//!   the body.
//!
//! A request with any `ce-*` header is treated as binary. If the body is itself
//! a complete structured CloudEvent, the structured body wins; this handles
//! Knative Kafka broker dispatches that duplicate the attributes in the body and
//! in `ce-*` headers. See [`CloudEvent`] for the exact rules.
//!
//! # Tagged routing
//!
//! With the default `macros` feature, the [`CloudEventTagged`] derive generates
//! `from_cloud_event_parts(event_type, data)` for enums. It is a router helper:
//! extract [`CloudEvent<serde_json::Value>`], then call it with the event type
//! and [`CloudEvent::data`].
//!
//! # Errors
//!
//! [`CloudEventError`] implements
//! [`IntoResponse`](axum::response::IntoResponse) and returns
//! `400 Bad Request` with a JSON body: `{"error":"..."}`.
//!
//! # Feature flags
//!
//! - `macros` (default): re-export the [`CloudEventTagged`] derive.
//!
//! Disable default features if you only need the extractor:
//!
//! ```toml
//! axum-cloudevents = { version = "0.1", default-features = false, registry = "kellnr" }
//! ```
//!
//! # Minimum supported Rust version
//!
//! Rust 1.92 or newer.
//!
//! # Body limits
//!
//! The extractor reads the full request body into memory. Configure Axum's
//! [`DefaultBodyLimit`](axum::extract::DefaultBodyLimit) for untrusted input.
//!
//! # Repository
//!
//! Source, examples, and guides:
//! <https://github.com/enchantednatures/axum-cloudevents>.

#![warn(missing_docs)]

pub use error::CloudEventError;
pub use extractor::CloudEvent;
pub use headers::*;
pub use metadata::CloudEventMetadata;

/// Derive macro for tagged CloudEvent routing.
#[cfg(feature = "macros")]
pub use axum_cloudevents_macros::CloudEventTagged;

mod error;
mod extractor;
mod headers;
mod metadata;
