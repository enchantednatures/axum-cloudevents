use crate::error::CloudEventError;
use crate::metadata::{CloudEventMetadata, Extensions};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct CloudEventBuilder<T> {
    id: Option<String>,
    source: Option<String>,
    r#type: Option<String>,
    spec_version: String,
    data_content_type: Option<String>,
    data_schema: Option<String>,
    subject: Option<String>,
    time: Option<String>,
    extensions: Extensions,
    data: Option<T>,
    _phantom: PhantomData<T>,
}

impl<T> CloudEventBuilder<T> {
    pub fn new() -> Self {
        Self {
            id: None,
            source: None,
            r#type: None,
            spec_version: DEFAULT_SPEC_VERSION.to_string(),
            data_content_type: None,
            data_schema: None,
            subject: None,
            time: None,
            extensions: Extensions::new(),
            data: None,
            _phantom: PhantomData,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn r#type(mut self, r#type: impl Into<String>) -> Self {
        self.r#type = Some(r#type.into());
        self
    }

    pub fn spec_version(mut self, spec_version: impl Into<String>) -> Self {
        self.spec_version = spec_version.into();
        self
    }

    pub fn data_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.data_content_type = Some(content_type.into());
        self
    }

    pub fn data_schema(mut self, schema: impl Into<String>) -> Self {
        self.data_schema = Some(schema.into());
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }

    pub fn time_now(mut self) -> Self {
        self.time = Some(format_timestamp_now());
        self
    }

    pub fn data(mut self, data: T) -> Self {
        self.data = Some(data);
        self
    }

    pub fn extension(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.extensions.insert(key, value);
        self
    }

    pub fn extensions(
        mut self,
        extensions: impl IntoIterator<Item = (impl Into<String>, impl Into<serde_json::Value>)>
    ) -> Self {
        for (key, value) in extensions {
            self.extensions.insert(key, value);
        }
        self
    }

    pub fn build(self) -> Result<crate::extractor::CloudEvent<T>, CloudEventError> {
        let id = self.id.ok_or(CloudEventError::MissingField("id"))?;
        let source = self.source.ok_or(CloudEventError::MissingField("source"))?;
        let r#type = self.r#type.ok_or(CloudEventError::MissingField("type"))?;
        let data = self.data.ok_or(CloudEventError::MissingField("data"))?;

        let metadata = CloudEventMetadata {
            id,
            source,
            r#type,
            spec_version: self.spec_version,
            data_content_type: self.data_content_type,
            data_schema: self.data_schema,
            subject: self.subject,
            time: self.time,
            extensions: self.extensions,
        };

        Ok(crate::extractor::CloudEvent::new(metadata, data))
    }
}

impl<T> Default for CloudEventBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

const DEFAULT_SPEC_VERSION: &str = "1.0";

pub fn format_timestamp_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    let nsecs = now.subsec_nanos();

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
        1970 + secs / 31536000,
        (secs / 2628000) % 12 + 1,
        (secs / 86400) % 30 + 1,
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60,
        nsecs
    )
}
