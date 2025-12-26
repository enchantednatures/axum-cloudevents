use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, header},
};
use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{CloudEventError, Result};
use crate::http::{is_structured_mode, parse_binary_mode_headers};
use crate::metadata::CloudEventMetadata;

const DEFAULT_SPEC_VERSION: &str = "1.0";

#[derive(Debug, Clone, Copy)]
pub struct CloudEventConfig {
    pub force_binary_mode: bool,
    pub strict_specversion: bool,
}

impl Default for CloudEventConfig {
    fn default() -> Self {
        Self {
            force_binary_mode: false,
            strict_specversion: true,
        }
    }
}

#[derive(Debug)]
pub struct CloudEvent<T> {
    pub metadata: CloudEventMetadata,
    pub data: T,
}

impl<T> CloudEvent<T> {
    pub fn new(metadata: CloudEventMetadata, data: T) -> Self {
        Self { metadata, data }
    }

    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    pub fn r#type(&self) -> &str {
        &self.metadata.r#type
    }

    pub fn source(&self) -> &str {
        &self.metadata.source
    }

    pub fn subject(&self) -> Option<&str> {
        self.metadata.subject.as_deref()
    }

    pub fn time(&self) -> Option<&str> {
        self.metadata.time.as_deref()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.metadata.data_content_type.as_deref()
    }

    pub fn into_data(self) -> T {
        self.data
    }

    pub fn into_parts(self) -> (CloudEventMetadata, T) {
        (self.metadata, self.data)
    }

    pub fn builder() -> crate::builder::CloudEventBuilder<T> {
        crate::builder::CloudEventBuilder::new()
    }
}

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

impl<S, T> FromRequest<S> for CloudEvent<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = CloudEventError;

    async fn from_request(req: Request<Body>, _state: &S) -> Result<Self> {
        let config = req
            .extensions()
            .get::<CloudEventConfig>()
            .copied()
            .unwrap_or_default();

        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v: &header::HeaderValue| v.to_str().ok());

        let is_structured = is_structured_mode(content_type) || (config.force_binary_mode && content_type.is_none());

        let event = if is_structured {
            extract_structured_mode(req, config).await?
        } else {
            extract_binary_mode(req, config).await?
        };

        Ok(event)
    }
}

async fn extract_structured_mode<T>(
    req: Request<Body>,
    config: CloudEventConfig,
) -> Result<CloudEvent<T>>
where
    T: DeserializeOwned + Send,
{
    let body_bytes: Bytes = Limited::new(req.into_body(), 64 * 1024 * 1024)
        .collect()
        .await
        .map_err(|e: axum::BoxError| CloudEventError::BodyError(e))?
        .to_bytes();

    let raw: RawCloudEvent = serde_json::from_slice(&body_bytes)?;

    validate_required_fields(&raw)?;
    validate_spec_version(&raw.spec_version, config.strict_specversion)?;

    let metadata = CloudEventMetadata {
        id: raw.id,
        source: raw.source,
        r#type: raw.ce_type,
        spec_version: raw.spec_version,
        data_content_type: raw.data_content_type,
        data_schema: raw.data_schema,
        subject: raw.subject,
        time: raw.time,
        extensions: crate::metadata::Extensions::new(),
    };

    let data = extract_data(raw.data)?;

    Ok(CloudEvent::new(metadata, data))
}

async fn extract_binary_mode<T>(
    req: Request<Body>,
    config: CloudEventConfig,
) -> Result<CloudEvent<T>>
where
    T: DeserializeOwned + Send,
{
    let metadata = parse_binary_mode_headers(req.headers())?;

    validate_required_fields_from_metadata(&metadata)?;
    validate_spec_version(&metadata.spec_version, config.strict_specversion)?;

    let body_bytes: Bytes = Limited::new(req.into_body(), 64 * 1024 * 1024)
        .collect()
        .await
        .map_err(|e: axum::BoxError| CloudEventError::BodyError(e))?
        .to_bytes();

    let data: T = if body_bytes.is_empty() {
        serde_json::from_value(Value::Null)?
    } else {
        let data_str = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| CloudEventError::InvalidUtf8(e.to_string()))?;

        if let Some(content_type) = &metadata.data_content_type {
            if content_type.contains("application/json") || content_type.contains("application/cloudevents+json") {
                serde_json::from_str(&data_str)?
            } else {
                serde_json::from_value(serde_json::Value::String(data_str))?
            }
        } else {
            serde_json::from_str(&data_str)?
        }
    };

    Ok(CloudEvent::new(metadata, data))
}

fn validate_required_fields(raw: &RawCloudEvent) -> Result<()> {
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

fn validate_required_fields_from_metadata(metadata: &CloudEventMetadata) -> Result<()> {
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

fn validate_spec_version(version: &str, strict: bool) -> Result<()> {
    if strict && version != "1.0" {
        return Err(CloudEventError::InvalidSpecVersion(version.to_string()));
    }
    if version.is_empty() {
        return Err(CloudEventError::MissingField("specversion"));
    }
    Ok(())
}

fn extract_data<T: DeserializeOwned>(data_value: Option<Value>) -> Result<T> {
    match data_value {
        Some(value) => serde_json::from_value(value)
            .map_err(CloudEventError::DataDeserialization),
        None => serde_json::from_value(Value::Null)
            .map_err(CloudEventError::DataDeserialization),
    }
}
