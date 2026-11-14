# axum-cloudevents

[![CI](https://github.com/enchantednatures/axum-cloudevents/actions/workflows/ci.yaml/badge.svg)](https://github.com/enchantednatures/axum-cloudevents/actions/workflows/ci.yaml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

CloudEvents 1.0 extractors for [Axum](https://github.com/tokio-rs/axum) 0.8+.

`axum-cloudevents` implements `FromRequest` for `CloudEvent<T>`, so a handler can
take a typed CloudEvent directly. Both structured
(`application/cloudevents+json`) and binary (`ce-*` headers) content modes are
supported, along with the `CloudEventTagged` derive for event-type routing.

> Not affiliated with or endorsed by the Axum project.

## Documentation

- [Documentation index](docs/README.md)
- [Usage guide](docs/USAGE.md) — content modes, metadata, tagged routing, errors, testing
- [Publishing](docs/PUBLISHING.md) — Kellnr releases, tokens, versioning
- [Changelog](CHANGELOG.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

## Crates

| Crate | Purpose |
| --- | --- |
| `axum-cloudevents` | Extractor, metadata, header constants, and the re-exported derive |
| `axum-cloudevents-macros` | `CloudEventTagged` derive macro |

## Installation

The crates are published to the private Kellnr registry
`https://crates.enchantednatures.com/api/v1/crates/`. Add the registry to
`.cargo/config.toml`:

```toml
[registries.kellnr]
index = "sparse+https://crates.enchantednatures.com/api/v1/crates/"
credential-provider = ["cargo:token"]
```

Authenticate once with `cargo login --registry kellnr`, or provide
`CARGO_REGISTRIES_KELLNR_TOKEN` in the environment. Then depend on the crate:

```toml
[dependencies]
axum-cloudevents = { version = "0.1", registry = "kellnr" }
```

The default `macros` feature enables the `CloudEventTagged` derive. Disable
default features if you only need the extractor:

```toml
axum-cloudevents = { version = "0.1", registry = "kellnr", default-features = false }
```

## Quick start

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

fn app() -> Router {
    Router::new().route("/events", post(handler))
}
```

Structured event:

```http
POST /events HTTP/1.1
Content-Type: application/cloudevents+json

{
  "specversion": "1.0",
  "id": "event-1",
  "source": "/pings",
  "type": "com.example.ping",
  "data": { "message": "hello" }
}
```

Binary event:

```http
POST /events HTTP/1.1
Content-Type: application/json
ce-specversion: 1.0
ce-id: event-2
ce-source: /pings
ce-type: com.example.ping

{ "message": "hello" }
```

## Content-mode detection

A request with any `ce-*` header is treated as binary. If the body is a JSON
object containing non-empty `specversion`, `id`, `source`, and `type`, the
structured body wins instead. This handles the Knative Kafka broker dispatcher,
which sends a structured body together with duplicate `ce-*` headers.

A request with `ce-source`/`ce-type` but no `ce-id` is treated as malformed
binary and returns a missing-header error rather than falling back to JSON.

## Tagged routing

`CloudEventTagged` generates an inherent
`from_cloud_event_parts(event_type, data)` method. It does **not** implement
`Deserialize` and does not hook into the extractor automatically. Extract the
raw payload and route it:

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
(`com.example.thing.created`) or the bare suffix (`created`).

## Errors

`CloudEventError` implements `IntoResponse` and returns `400 Bad Request` with a
JSON body:

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

The extractor reads the full body into memory; configure Axum's
`DefaultBodyLimit` for untrusted input.

## Examples

```bash
cargo run -p axum-cloudevents --example simple
cargo run -p axum-cloudevents --example tagged
```

`simple` demonstrates both content modes; `tagged` demonstrates the manual
`CloudEventTagged` routing pattern.

## Compatibility

- Axum 0.8
- CloudEvents 1.0
- Rust 1.92 or newer

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

The workspace `.cargo/config.toml` patches the macros dependency to the local
member, so development and CI do not need a Kellnr token. Publishing still
resolves the dependency from Kellnr.

## Publishing

Versions are centralized in `[workspace.package]`. To release:

1. Update `version` in `Cargo.toml`.
2. Update `CHANGELOG.md`.
3. Commit, then push a `crates-vX.Y.Z` tag.
4. The publish workflow verifies the tag matches both crates, publishes
   `axum-cloudevents-macros` first, then `axum-cloudevents`.

The workflow requires the `KELLNR_PUBLISH_TOKEN` repository secret. See
[docs/PUBLISHING.md](docs/PUBLISHING.md) for the full release and token process.

## License

MIT. See [LICENSE](LICENSE).
