pub use axum_cloudevents_core::{
    CloudEvent as CoreCloudEvent, CloudEventError, CloudEventMetadata, Extensions,
};
pub use axum_cloudevents_macros::CloudEventTagged;

use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, header},
};
use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use serde_json::Value;

use axum_cloudevents_core::CloudEventConfig;

const DEFAULT_SPEC_VERSION: &str = "1.0";

/// Trait for types that can be constructed from a CloudEvent type string and data
pub trait CloudEventTagged: Sized + Send {
    /// The event type prefix (e.g., "com.example.user" for "com.example.user.created")
    const TYPE_PREFIX: &'static str;

    /// Map from CloudEvent type string and data to Self
    fn from_cloud_event_parts(event_type: &str, data: Value) -> Result<Self, String>;
}

/// Raw CloudEvent for parsing (structured mode)
#[derive(serde::Deserialize)]
struct RawCloudEvent {
    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "source")]
    source: String,

    #[serde(rename = "type")]
    ce_type: String,

    #[serde(rename = "specversion", default = "default_spec_version")]
    spec_version: String,

    #[serde(rename = "datacontenttype", default)]
    data_content_type: Option<String>,

    #[serde(rename = "dataschema", default)]
    data_schema: Option<String>,

    #[serde(rename = "subject", default)]
    subject: Option<String>,

    #[serde(rename = "time", default)]
    time: Option<String>,

    #[serde(rename = "data", default)]
    data: Option<Value>,
}

fn default_spec_version() -> String {
    DEFAULT_SPEC_VERSION.to_string()
}

/// CloudEvent extractor for tagged enum types
///
/// # Examples
///
/// ```ignore
/// use axum_cloudevents_tagged::{CloudEventTagged, CloudEvent};
/// use serde::Deserialize;
///
/// #[derive(Deserialize, CloudEventTagged)]
/// #[cloud_event_prefix("com.example.payment")]
/// enum PaymentEvent {
///     #[cloud_event_variant("processed")]
///     Processed(PaymentProcessed),
///
///     #[cloud_event_variant("failed")]
///     Failed(PaymentFailed),
///
///     #[cloud_event_catchall]
///     Unknown(serde_json::Value),
/// }
///
/// async fn handler(event: CloudEvent<PaymentEvent>) -> &'static str {
///     match event.data {
///         PaymentEvent::Processed(p) => { /* ... */ }
///         PaymentEvent::Failed(f) => { /* ... */ }
///         PaymentEvent::Unknown(_) => { /* ... */ }
///     }
///     "OK"
/// }
/// ```
pub struct CloudEvent<T: CloudEventTagged> {
    /// CloudEvent attributes/metadata
    pub metadata: CloudEventMetadata,

    /// Tagged enum variant with payload
    pub data: T,
}

impl<T: CloudEventTagged> CloudEvent<T> {
    pub fn new(metadata: CloudEventMetadata, data: T) -> Self {
        Self { metadata, data }
    }

    /// Access event ID
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// Access event type
    pub fn r#type(&self) -> &str {
        &self.metadata.r#type
    }

    /// Access event source
    pub fn source(&self) -> &str {
        &self.metadata.source
    }

    /// Access event subject
    pub fn subject(&self) -> Option<&str> {
        self.metadata.subject.as_deref()
    }

    /// Access event timestamp
    pub fn time(&self) -> Option<&str> {
        self.metadata.time.as_deref()
    }

    /// Access data content type
    pub fn content_type(&self) -> Option<&str> {
        self.metadata.data_content_type.as_deref()
    }
}

impl<S, T> FromRequest<S> for CloudEvent<T>
where
    T: CloudEventTagged + Send + 'static,
    S: Send + Sync,
{
    type Rejection = CloudEventError;

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self, CloudEventError> {
        let config = req
            .extensions()
            .get::<CloudEventConfig>()
            .copied()
            .unwrap_or_default();

        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v: &header::HeaderValue| v.to_str().ok());

        let is_structured = axum_cloudevents_core::http::is_structured_mode(content_type)
            || (config.force_binary_mode && content_type.is_none());

        if is_structured {
            extract_tagged_structured(req, config).await
        } else {
            extract_tagged_binary(req, config).await
        }
    }
}

async fn extract_tagged_structured<T>(
    req: Request<Body>,
    config: CloudEventConfig,
) -> Result<CloudEvent<T>, CloudEventError>
where
    T: CloudEventTagged + Send + 'static,
{
    let body_bytes: Bytes = Limited::new(req.into_body(), 64 * 1024 * 1024)
        .collect()
        .await
        .map_err(|e: axum::BoxError| CloudEventError::BodyError(e))?
        .to_bytes();

    let raw: RawCloudEvent = serde_json::from_slice(&body_bytes)?;

    validate_required_fields(&raw)?;
    validate_spec_version(&raw.spec_version, config.strict_specversion)?;

    let data = raw.data.unwrap_or_else(|| Value::Null);

    let tagged_data = T::from_cloud_event_parts(&raw.ce_type, data)
        .map_err(|e| CloudEventError::NoMatchingVariant(e))?;

    let metadata = CloudEventMetadata {
        id: raw.id,
        source: raw.source,
        r#type: raw.ce_type,
        spec_version: raw.spec_version,
        data_content_type: raw.data_content_type,
        data_schema: raw.data_schema,
        subject: raw.subject,
        time: raw.time,
        extensions: Extensions::new(),
    };

    Ok(CloudEvent::new(metadata, tagged_data))
}

async fn extract_tagged_binary<T>(
    req: Request<Body>,
    config: CloudEventConfig,
) -> Result<CloudEvent<T>, CloudEventError>
where
    T: CloudEventTagged + Send + 'static,
{
    let metadata = axum_cloudevents_core::http::parse_binary_mode_headers(req.headers())?;

    validate_required_fields_from_metadata(&metadata)?;
    validate_spec_version(&metadata.spec_version, config.strict_specversion)?;

    let body_bytes: Bytes = Limited::new(req.into_body(), 64 * 1024 * 1024)
        .collect()
        .await
        .map_err(|e: axum::BoxError| CloudEventError::BodyError(e))?
        .to_bytes();

    let data: Value = if body_bytes.is_empty() {
        Value::Null
    } else {
        let data_str = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| CloudEventError::InvalidUtf8(e.to_string()))?;

        if let Some(content_type) = &metadata.data_content_type {
            if content_type.contains("application/json")
                || content_type.contains("application/cloudevents+json")
            {
                serde_json::from_str(&data_str)?
            } else {
                serde_json::from_value(Value::String(data_str))?
            }
        } else {
            serde_json::from_str(&data_str)?
        }
    };

    let tagged_data = T::from_cloud_event_parts(&metadata.r#type, data)
        .map_err(|e| CloudEventError::NoMatchingVariant(e))?;

    Ok(CloudEvent::new(metadata, tagged_data))
}

fn validate_required_fields(raw: &RawCloudEvent) -> Result<(), CloudEventError> {
    if raw.id.is_empty() {
        return Err(CloudEventError::MissingField("id"));
    }
    if raw.source.is_empty() {
        return Err(CloudEventError::MissingField("source"));
    }
    if raw.ce_type.is_empty() {
        return Err(CloudEventError::MissingField("type"));
    }
    Ok(())
}

fn validate_required_fields_from_metadata(
    metadata: &CloudEventMetadata,
) -> Result<(), CloudEventError> {
    if metadata.id.is_empty() {
        return Err(CloudEventError::MissingField("id"));
    }
    if metadata.source.is_empty() {
        return Err(CloudEventError::MissingField("source"));
    }
    if metadata.r#type.is_empty() {
        return Err(CloudEventError::MissingField("type"));
    }
    Ok(())
}

fn validate_spec_version(
    version: &str,
    strict: bool,
) -> Result<(), CloudEventError> {
    if strict && version != "1.0" {
        return Err(CloudEventError::InvalidSpecVersion(version.to_string()));
    }
    if version.is_empty() {
        return Err(CloudEventError::MissingField("specversion"));
    }
    Ok(())
}
