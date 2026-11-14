//! Minimal typed CloudEvent handler.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p axum-cloudevents --example simple
//! ```
//!
//! Then send a structured event:
//!
//! ```bash
//! curl -X POST http://127.0.0.1:3000/events \
//!   -H 'content-type: application/cloudevents+json' \
//!   -d '{"specversion":"1.0","id":"evt-1","source":"/examples/simple","type":"com.example.ping","data":{"message":"hello"}}'
//! ```
//!
//! Or a binary event:
//!
//! ```bash
//! curl -X POST http://127.0.0.1:3000/events \
//!   -H 'content-type: application/json' \
//!   -H 'ce-specversion: 1.0' \
//!   -H 'ce-id: evt-2' \
//!   -H 'ce-source: /examples/simple' \
//!   -H 'ce-type: com.example.ping' \
//!   -d '{"message":"hello"}'
//! ```

use axum::{routing::post, Json, Router};
use axum_cloudevents::CloudEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Ping {
    message: String,
}

#[derive(Debug, Serialize)]
struct Pong {
    reply: String,
    event_id: String,
}

async fn handle(event: CloudEvent<Ping>) -> Json<Pong> {
    Json(Pong {
        reply: format!("pong: {}", event.data.message),
        event_id: event.id().to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/events", post(handle));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
