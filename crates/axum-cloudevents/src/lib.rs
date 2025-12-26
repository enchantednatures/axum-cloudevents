//! Ergonomic CloudEvents extractors for Axum 0.8+
//!
//! This crate provides a highly ergonomic interface for working with CloudEvents
//! in Axum applications, with support for both binary and structured content modes.
//!
//! # Features
//!
//! - ✅ CloudEvents v1.0.2 compliant
//! - ✅ Binary and Structured content modes
//! - ✅ Type-safe event extraction
//! - ✅ Tagged enum routing for multi-event handlers
//! - ✅ Extension attributes support
//! - ✅ Response builder API
//! - ✅ Optional tracing integration
//! - ✅ Knative-optimized
//!
//! # Modules
//!
//! - [`CloudEvent`] - Main extractor for typed events
//! - [`CloudEventMetadata`] - Event metadata/attributes
//! - [`CloudEventConfig`] - Configuration for extraction
//!
//! # Feature Flags
//!
//! - `tagged` - Enable tagged enum routing (enabled by default)
//! - `tracing` - Enable tracing integration

pub use axum_cloudevents_core::{
    CloudEvent, CloudEventConfig, CloudEventError, CloudEventMetadata,
    Extensions, CloudEventBuilder, CloudEventResponse,
    IntoCloudEventResponse, cloud_event, Result,
    builder::format_timestamp_now,
};

#[cfg(feature = "tagged")]
pub use axum_cloudevents_tagged::CloudEventTagged;

#[cfg(feature = "tagged")]
pub use axum_cloudevents_macros::CloudEventTagged as CloudEventTaggedDerive;

#[cfg(feature = "tagged")]
pub use axum_cloudevents_macros::IntoCloudEventResponse as IntoCloudEventResponseDerive;
