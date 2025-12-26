use axum::{
    http::header,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;

use crate::extractor::CloudEvent;

pub trait IntoCloudEventResponse {
    fn cloud_event_type() -> &'static str;

    fn cloud_event_source() -> &'static str {
        "/axum-cloudevents"
    }

    fn cloud_event_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn cloud_event_time(&self) -> Option<String> {
        Some(crate::builder::format_timestamp_now())
    }

    fn into_cloud_event(self) -> CloudEvent<Self>
    where
        Self: Sized + Serialize,
    {
        let id = self.cloud_event_id();
        let source = Self::cloud_event_source().to_string();
        let r#type = Self::cloud_event_type().to_string();
        let time = self.cloud_event_time();

        let mut metadata = crate::metadata::CloudEventMetadata::new(id, source, r#type);
        if let Some(t) = time {
            metadata.time = Some(t);
        }

        CloudEvent::new(metadata, self)
    }
}

pub struct CloudEventResponse<T>(pub CloudEvent<T>)
where
    T: Send + Serialize;

impl<T> IntoResponse for CloudEventResponse<T>
where
    T: Send + Serialize,
{
    fn into_response(self) -> Response {
        let mut metadata_json = serde_json::to_value(&self.0.metadata)
            .unwrap_or_else(|_| serde_json::Value::Object(Default::default()));

        if let serde_json::Value::Object(ref mut map) = metadata_json {
            map.insert(
                "data".to_string(),
                serde_json::to_value(&self.0.data).unwrap_or(serde_json::Value::Null),
            );
        }

        (
            [(header::CONTENT_TYPE, "application/cloudevents+json")],
            Json(metadata_json),
        )
            .into_response()
    }
}

impl<T> IntoResponse for CloudEvent<T>
where
    T: Send + Serialize,
{
    fn into_response(self) -> Response {
        CloudEventResponse(self).into_response()
    }
}

pub fn cloud_event<T>(
    event_type: &str,
    source: &str,
    data: T,
) -> CloudEvent<T>
where
    T: Send + Serialize,
{
    CloudEvent::builder()
        .id(uuid::Uuid::new_v4().to_string())
        .source(source)
        .r#type(event_type)
        .time_now()
        .data(data)
        .build()
        .unwrap()
}
