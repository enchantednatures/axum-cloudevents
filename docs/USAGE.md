# Usage

This guide covers the extractor, content modes, typed payloads, tagged routing,
error handling, and testing.

## Installation

The crates are published to the private Kellnr registry
`https://crates.enchantednatures.com/api/v1/crates/`. Add the registry to
`.cargo/config.toml`:

```toml
[registries.kellnr]
index = "sparse+https://crates.enchantednatures.com/api/v1/crates/"
credential-provider = ["cargo:token"]
```

Authenticate once:

```bash
cargo login --registry kellnr
```

Or set the token in the environment:

```bash
export CARGO_REGISTRIES_KELLNR_TOKEN=...
```

Then add the dependency:

```toml
[dependencies]
axum-cloudevents = { version = "0.1", registry = "kellnr" }
```

The default `macros` feature re-exports the `CloudEventTagged` derive. Disable
default features if you only need the extractor:

```toml
axum-cloudevents = { version = "0.1", registry = "kellnr", default-features = false }
```

## Basic handler

`CloudEvent<T>` implements `FromRequest`, so a handler can accept it directly:

```rust
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

async fn handler(event: CloudEvent<Ping>) -> Json<Pong> {
    Json(Pong {
        reply: format!("pong: {}", event.data.message),
        event_id: event.id().to_string(),
    })
}

let app = Router::new().route("/events", post(handler));
```

`T` can be any `serde::de::DeserializeOwned + Send` type, including
`serde_json::Value` when the payload shape varies.

## Content modes

### Structured

The body is a JSON CloudEvent envelope. The extractor reads the attributes and
deserializes `data` into `T`:

```http
POST /events HTTP/1.1
Content-Type: application/cloudevents+json

{
  "specversion": "1.0",
  "id": "event-1",
  "source": "/api/v1/pings",
  "type": "com.example.ping",
  "data": { "message": "hello" }
}
```

### Binary

The attributes are `ce-*` headers and the body is the payload:

```http
POST /events HTTP/1.1
Content-Type: application/json
ce-specversion: 1.0
ce-id: event-2
ce-source: /api/v1/pings
ce-type: com.example.ping

{ "message": "hello" }
```

## Content-mode detection

The extractor uses these rules:

1. If the request has any `ce-*` header, it is a binary candidate.
2. If the body is a JSON object with non-empty `specversion`, `id`, `source`,
   and `type`, the body wins and is parsed as structured.
3. Otherwise, the request is parsed as binary.

Rule 2 handles the Knative Kafka broker dispatcher, which sends a complete
structured CloudEvent body **and** duplicates the attributes in `ce-*` headers.
Treating that request as binary would try to deserialize the envelope into the
payload type and fail.

If a request has `ce-source`/`ce-type` but no `ce-id`, it is treated as a
malformed binary event and returns `400` with a missing-header error rather
than falling back to JSON parsing.

## Metadata

`CloudEvent<T>` exposes the metadata and convenience accessors:

```rust
let id = event.id();
let event_type = event.r#type();
let source = event.source();
let subject = event.subject();
let time = event.time();
let content_type = event.content_type();
let spec_version = &event.metadata.spec_version;
```

`CloudEventMetadata::type_matches(prefix)` checks whether the event type starts
with a prefix, for example:

```rust
if event.metadata.type_matches("com.example.payment.") {
    // route payment events
}
```

## Tagged enum routing

`CloudEventTagged` generates an inherent
`from_cloud_event_parts(event_type, data)` method. It does **not** implement
`Deserialize` or hook into the extractor automatically. Extract the raw data and
route it:

```rust
use axum::{http::StatusCode, Json};
use axum_cloudevents::{CloudEvent, CloudEventTagged};
use serde::Deserialize;

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

async fn handler(
    event: CloudEvent<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let (metadata, data) = event.into_parts();
    let routed = ThingEvent::from_cloud_event_parts(&metadata.r#type, data)
        .map_err(|message| (StatusCode::BAD_REQUEST, message))?;

    let id = match routed {
        ThingEvent::Created(payload) => payload.id,
        ThingEvent::Deleted(payload) => payload.id,
    };

    Ok(Json(serde_json::json!({ "id": id, "event_id": metadata.id })))
}
```

The generated method accepts either the fully prefixed type
(`com.example.thing.created`) or the bare suffix (`created`). A variant without
a matching event type returns `Err(String)` describing the unknown type.

## Errors

`CloudEventError` implements `IntoResponse`. Every variant maps to
`400 Bad Request` with a JSON body:

```json
{ "error": "Missing header: ce-id" }
```

| Variant | Cause |
| --- | --- |
| `BodyRead` | The request body could not be read |
| `InvalidJson` | The body was not JSON or did not match the payload type |
| `MissingField` | A required structured envelope field was missing or empty |
| `MissingHeader` | A required `ce-*` header was missing or not valid UTF-8 |
| `UnknownEventType` | A tagged router did not recognize the event type |

The extractor logs the failure with `tracing::error!` before returning the
response.

## Body limits

The extractor reads the full body into memory. Axum's
`DefaultBodyLimit` applies; configure it for untrusted input:

```rust
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    .route("/events", post(handler))
    .layer(DefaultBodyLimit::max(1024 * 1024));
```

## Testing

`axum-test` works with the extractor. Send a structured event:

```rust
use axum::Router;
use axum_cloudevents::CloudEvent;

#[tokio::test]
async fn accepts_a_structured_event() {
    let app = Router::new().route("/events", axum::routing::post(handler));
    let server = axum_test::TestServer::new(app).unwrap();

    let response = server
        .post("/events")
        .content_type("application/cloudevents+json")
        .json(&serde_json::json!({
            "specversion": "1.0",
            "id": "evt-1",
            "source": "/tests",
            "type": "com.example.ping",
            "data": { "message": "hello" }
        }))
        .await;

    response.assert_status_ok();
}
```

Binary mode works the same way with `ce-*` headers:

```rust
let response = server
    .post("/events")
    .add_header("ce-id", "evt-2")
    .add_header("ce-source", "/tests")
    .add_header("ce-type", "com.example.ping")
    .add_header("ce-specversion", "1.0")
    .json(&serde_json::json!({ "message": "hello" }))
    .await;
```

## Examples

The workspace includes runnable examples:

```bash
cargo run -p axum-cloudevents --example simple
cargo run -p axum-cloudevents --example tagged
```

`simple` demonstrates both content modes; `tagged` demonstrates the
`CloudEventTagged` manual routing pattern.

## Compatibility

- Axum 0.8
- CloudEvents 1.0
- Rust 1.92 or newer

## Feature flags

| Feature | Default | Description |
| --- | --- | --- |
| `macros` | yes | Re-export the `CloudEventTagged` derive |
