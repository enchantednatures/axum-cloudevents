use crate::error::{CloudEventError, Result};
use axum::http::{HeaderMap, HeaderValue, header};
use crate::metadata::{CloudEventMetadata, Extensions};
use percent_encoding::{percent_decode, percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde_json::Value;

/// Check if content type indicates structured mode
pub fn is_structured_mode(content_type: Option<&str>) -> bool {
    content_type
        .map(|ct| ct.starts_with("application/cloudevents+"))
        .unwrap_or(false)
}

/// Parse CloudEvent from HTTP headers (binary mode)
pub fn parse_binary_mode_headers(headers: &HeaderMap) -> Result<CloudEventMetadata> {
    let mut metadata = CloudEventMetadata {
        id: extract_required_header(headers, "ce-id")?,
        source: extract_required_header(headers, "ce-source")?,
        r#type: extract_required_header(headers, "ce-type")?,
        spec_version: extract_required_header(headers, "ce-specversion")?,
        data_content_type: None,
        data_schema: None,
        subject: None,
        time: None,
        extensions: Extensions::new(),
    };

    metadata.subject = extract_optional_header(headers, "ce-subject");
    metadata.time = extract_optional_header(headers, "ce-time");
    metadata.data_schema = extract_optional_header(headers, "ce-dataschema");

    metadata.data_content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        if name_str.starts_with("ce-") && !is_standard_attribute(name_str) {
            let ext_name = &name_str[3..];
            let decoded_value = percent_decode_header(value)?;
            metadata.extensions.insert(ext_name, decoded_value);
        }
    }

    Ok(metadata)
}

fn is_standard_attribute(header_name: &str) -> bool {
    matches!(
        header_name,
        "ce-id" | "ce-source" | "ce-type" | "ce-specversion"
            | "ce-datacontenttype" | "ce-dataschema" | "ce-subject" | "ce-time"
    )
}

fn extract_required_header(headers: &HeaderMap, name: &str) -> Result<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| percent_decode_header_value(s))
        .ok_or_else(|| CloudEventError::MissingHeader("Required header not found"))
}

fn extract_optional_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| percent_decode_header_value(s))
}

fn percent_decode_header_value(value: &str) -> String {
    let decoded_bytes = percent_decode(value.as_bytes())
        .collect::<Vec<u8>>();

    String::from_utf8_lossy(&decoded_bytes).to_string()
}

fn percent_decode_header(value: &HeaderValue) -> Result<Value> {
    let value_str = value.to_str()
        .map_err(|_| CloudEventError::InvalidHeaderValue)?;

    let decoded = percent_decode_header_value(value_str);

    serde_json::from_str(&decoded)
        .or_else(|_| Ok(Value::String(decoded)))
}

/// Custom percent-encoding set for CloudEvents HTTP headers
const CLOUD_EVENTS_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

/// Percent-encode value for HTTP header per CloudEvents spec
pub fn percent_encode_for_header(value: &str) -> String {
    percent_encode(value.as_bytes(), CLOUD_EVENTS_ENCODE_SET)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_is_structured_mode() {
        assert!(is_structured_mode(Some("application/cloudevents+json")));
        assert!(is_structured_mode(Some("application/cloudevents+json; charset=utf-8")));
        assert!(!is_structured_mode(Some("application/json")));
        assert!(!is_structured_mode(None));
    }

    #[test]
    fn test_percent_decode_header_value() {
        assert_eq!(
            percent_decode_header_value("Hello%20World"),
            "Hello World"
        );
        assert_eq!(
            percent_decode_header_value("test%3A123"),
            "test:123"
        );
        assert_eq!(
            percent_decode_header_value("normal-text"),
            "normal-text"
        );
    }

    #[test]
    fn test_percent_encode_for_header() {
        let encoded = percent_encode_for_header("Hello World");
        assert_eq!(encoded, "Hello%20World");

        let encoded = percent_encode_for_header("test:123");
        assert_eq!(encoded, "test%3A123");

        let encoded = percent_encode_for_header("normal-text");
        assert_eq!(encoded, "normal-text");
    }

    #[test]
    fn test_parse_binary_mode_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("ce-id", HeaderValue::from_static("test-123"));
        headers.insert("ce-source", HeaderValue::from_static("https://example.com"));
        headers.insert("ce-type", HeaderValue::from_static("com.example.test"));
        headers.insert("ce-specversion", HeaderValue::from_static("1.0"));
        headers.insert("ce-customext", HeaderValue::from_static("custom-value"));

        let metadata = parse_binary_mode_headers(&headers).unwrap();

        assert_eq!(metadata.id, "test-123");
        assert_eq!(metadata.source, "https://example.com");
        assert_eq!(metadata.r#type, "com.example.test");
        assert_eq!(metadata.spec_version, "1.0");
        assert_eq!(metadata.extensions.get("customext"), Some(&Value::String("custom-value".to_string())));
    }

    #[test]
    fn test_parse_binary_mode_headers_with_encoding() {
        let mut headers = HeaderMap::new();
        headers.insert("ce-id", HeaderValue::from_static("test-456"));
        headers.insert("ce-source", HeaderValue::from_static("/source"));
        headers.insert("ce-type", HeaderValue::from_static("test.event"));
        headers.insert("ce-specversion", HeaderValue::from_static("1.0"));
        headers.insert("ce-message", HeaderValue::from_static("Hello%20World"));

        let metadata = parse_binary_mode_headers(&headers).unwrap();

        assert_eq!(metadata.extensions.get("message"), Some(&Value::String("Hello World".to_string())));
    }
}
