//! CloudEvents 1.0 header names and media types.

/// `ce-id` header name.
pub const CE_ID: &str = "ce-id";

/// `ce-source` header name.
pub const CE_SOURCE: &str = "ce-source";

/// `ce-type` header name.
pub const CE_TYPE: &str = "ce-type";

/// `ce-specversion` header name.
pub const CE_SPECVERSION: &str = "ce-specversion";

/// `ce-datacontenttype` header name.
pub const CE_DATACONTENTTYPE: &str = "ce-datacontenttype";

/// `ce-dataschema` header name.
pub const CE_DATASCHEMA: &str = "ce-dataschema";

/// `ce-subject` header name.
pub const CE_SUBJECT: &str = "ce-subject";

/// `ce-time` header name.
pub const CE_TIME: &str = "ce-time";

/// Media type used by structured-mode CloudEvents bodies.
pub const CLOUDEVENTS_JSON_CONTENT_TYPE: &str = "application/cloudevents+json";
