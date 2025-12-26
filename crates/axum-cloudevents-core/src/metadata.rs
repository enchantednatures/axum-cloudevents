use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudEventMetadata {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "source")]
    pub source: String,

    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "specversion")]
    pub spec_version: String,

    #[serde(rename = "datacontenttype", skip_serializing_if = "Option::is_none")]
    pub data_content_type: Option<String>,

    #[serde(rename = "dataschema", skip_serializing_if = "Option::is_none")]
    pub data_schema: Option<String>,

    #[serde(rename = "subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    #[serde(flatten)]
    pub extensions: Extensions,
}

impl CloudEventMetadata {
    pub fn new(id: impl Into<String>, source: impl Into<String>, r#type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            r#type: r#type.into(),
            spec_version: "1.0".to_string(),
            data_content_type: None,
            data_schema: None,
            subject: None,
            time: None,
            extensions: Extensions::new(),
        }
    }

    pub fn type_matches(&self, prefix: &str) -> bool {
        self.r#type.starts_with(prefix)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    pub fn time(&self) -> Option<&str> {
        self.time.as_deref()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.data_content_type.as_deref()
    }

    pub fn data_schema(&self) -> Option<&str> {
        self.data_schema.as_deref()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Extensions(pub HashMap<String, serde_json::Value>);

impl Extensions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn get_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<Result<T, serde_json::Error>> {
        self.0.get(key).map(|v| serde_json::from_value(v.clone()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
