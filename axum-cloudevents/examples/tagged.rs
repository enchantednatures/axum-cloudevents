//! Tagged enum routing with `CloudEventTagged`.
//!
//! The derive generates `from_cloud_event_parts`; it does not implement
//! `Deserialize` or hook into the extractor automatically. Extract the raw
//! event data and route it manually.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p axum-cloudevents --example tagged
//! ```
//!
//! Then send an event:
//!
//! ```bash
//! curl -X POST http://127.0.0.1:3001/events \
//!   -H 'content-type: application/cloudevents+json' \
//!   -d '{"specversion":"1.0","id":"evt-1","source":"/examples/tagged","type":"com.example.thing.created","data":{"id":"thing-1"}}'
//! ```

use axum::{http::StatusCode, routing::post, Json, Router};
use axum_cloudevents::{CloudEvent, CloudEventTagged};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Created {
    id: String,
}

#[derive(Debug, Deserialize)]
struct Deleted {
    id: String,
}

#[derive(Debug, CloudEventTagged)]
#[cloud_event_prefix = "com.example.thing"]
enum ThingEvent {
    #[cloud_event_type("created")]
    Created(Created),
    #[cloud_event_type("deleted")]
    Deleted(Deleted),
}

#[derive(Debug, Serialize)]
struct Ack {
    event: &'static str,
    id: String,
    event_id: String,
}

async fn handle(event: CloudEvent<serde_json::Value>) -> Result<Json<Ack>, (StatusCode, String)> {
    let (metadata, data) = event.into_parts();
    let event_id = metadata.id.clone();
    let routed = ThingEvent::from_cloud_event_parts(&metadata.r#type, data)
        .map_err(|message| (StatusCode::BAD_REQUEST, message))?;

    match routed {
        ThingEvent::Created(payload) => Ok(Json(Ack {
            event: "created",
            id: payload.id,
            event_id,
        })),
        ThingEvent::Deleted(payload) => Ok(Json(Ack {
            event: "deleted",
            id: payload.id,
            event_id,
        })),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/events", post(handle));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;

    println!("listening on http://127.0.0.1:3001");
    axum::serve(listener, app).await?;

    Ok(())
}
