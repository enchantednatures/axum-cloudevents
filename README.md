# axum-cloudevents

CloudEvents 1.0 extractors for [Axum](https://github.com/tokio-rs/axum) 0.8+.

`axum-cloudevents` implements `FromRequest` for `CloudEvent<T>`, so a handler can
take a typed CloudEvent directly. Both structured (`application/cloudevents+json`)
and binary (`ce-*` headers) content modes are supported, along with the
`CloudEventTagged` derive for routing event types to enum variants.

> Not affiliated with or endorsed by the Axum project.

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

## Tagged enum routing

```rust
use axum_cloudevents::CloudEventTagged;
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

let event = ThingEvent::from_cloud_event_parts(
    "com.example.thing.created",
    serde_json::json!({ "id": "thing-1" }),
)
.expect("known event type");
```

The derive accepts either the fully prefixed type (`com.example.thing.created`)
or the bare suffix (`created`).

## Metadata

`CloudEvent<T>` exposes `metadata` plus convenience accessors:

```rust
event.id();
event.r#type();
event.source();
event.subject();
event.time();
event.content_type();
```

Header-name constants (`CE_ID`, `CE_SOURCE`, `CE_TYPE`, `CE_SPECVERSION`,
`CE_DATACONTENTTYPE`, `CE_DATASCHEMA`, `CE_SUBJECT`, `CE_TIME`) are available
from the crate root, as is `CLOUDEVENTS_JSON_CONTENT_TYPE`.

## Feature flags

| Feature | Default | Description |
| --- | --- | --- |
| `macros` | yes | Re-export `CloudEventTagged` from `axum-cloudevents-macros` |

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

The workspace `.cargo/config.toml` patches the macros dependency to the local
workspace member, so development and CI do not need a Kellnr token. Publishing
still resolves the dependency from Kellnr.

## Publishing

Versions are centralized in `[workspace.package]`. To release:

1. Update `version` in `Cargo.toml`.
2. Update the consuming project's dependency range if the minor version changed.
3. Commit, then push a `crates-vX.Y.Z` tag.
4. The publish workflow verifies the tag matches both crates, publishes
   `axum-cloudevents-macros` first, then `axum-cloudevents`.

The workflow requires the `KELLNR_PUBLISH_TOKEN` repository secret. A manual
`workflow_dispatch` run supports a dry-run mode.

## License

MIT. See [LICENSE](LICENSE).
