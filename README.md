# axum-cloudevents

Ergonomic CloudEvents extractors for Axum 0.8+

## Features

- ✅ CloudEvents v1.0.2 compliant
- ✅ Binary and Structured content modes
- ✅ Type-safe event extraction
- ✅ Tagged enum routing for multi-event handlers
- ✅ Extension attributes support
- ✅ Response builder API
- ✅ Optional tracing integration
- ✅ Comprehensive test coverage
- ✅ Full examples for all features

## Installation

```toml
[dependencies]
axum-cloudevents = "0.1"
```

## Quick Start

### Simple Event Handler

```rust,no_run
use axum::{Router, routing::post};
use axum_cloudevents::CloudEvent;
use serde::Deserialize;

#[derive(Deserialize)]
struct MyEvent {
    message: String,
}

async fn handler(event: CloudEvent<MyEvent>) -> &'static str {
    println!("Received: {}", event.data.message);
    "OK"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/events", post(handler));
    // ... serve app
}
```

### Tagged Enum Routing

```rust,no_run
use axum_cloudevents::{CloudEvent, CloudEventTagged};
use serde::Deserialize;

#[derive(Deserialize, CloudEventTagged)]
#[cloud_event_prefix("com.example.payment")]
enum PaymentEvent {
    #[cloud_event_variant("processed")]
    Processed(PaymentProcessed),

    #[cloud_event_variant("failed")]
    Failed(PaymentFailed),

    #[cloud_event_catchall]
    Unknown(serde_json::Value),
}

async fn handler(event: CloudEvent<PaymentEvent>) -> &'static str {
    match event.data {
        PaymentEvent::Processed(p) => { /* ... */ }
        PaymentEvent::Failed(f) => { /* ... */ }
        PaymentEvent::Unknown(_) => { /* log unknown */ }
    }
    "OK"
}
```

### Event Response Builder

```rust,no_run
use axum_cloudevents::{CloudEvent, cloud_event, IntoCloudEventResponse};
use serde::Serialize;

#[derive(Serialize, IntoCloudEventResponse)]
#[cloud_event(type = "com.example.user.created", source = "/users")]
struct UserCreatedEvent {
    user_id: String,
    email: String,
}

async fn handler() -> UserCreatedEvent {
    UserCreatedEvent {
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
    }
}

// Or use builder directly
async fn manual_builder() -> CloudEvent<serde_json::Value> {
    CloudEvent::builder()
        .id("event-123")
        .source("/myapp")
        .r#type("com.example.event")
        .time_now()
        .extension("tenant-id", "acme-corp")
        .data(serde_json::json!({ "message": "Hello" }))
        .build()
        .unwrap()
}
```

## Content Modes

### Structured Mode (JSON)

```http
POST /events HTTP/1.1
Content-Type: application/cloudevents+json

{
  "specversion": "1.0",
  "type": "com.example.event",
  "source": "/myapp",
  "id": "event-123",
  "data": {
    "message": "Hello"
  }
}
```

### Binary Mode (Headers + Body)

```http
POST /events HTTP/1.1
Content-Type: application/json
ce-specversion: 1.0
ce-type: com.example.event
ce-source: /myapp
ce-id: event-123
ce-customext: custom-value

{
  "message": "Hello"
}
```

## Configuration

### Force Binary Mode

```rust
use axum::{extract::State, Router};
use axum_cloudevents::{CloudEvent, CloudEventConfig};

#[derive(Clone)]
struct AppState {
    config: CloudEventConfig,
}

async fn handler(
    State(_state): State<AppState>,
    event: CloudEvent<MyData>,
) -> &'static str {
    // ... handle event
    "OK"
}

#[tokio::main]
async fn main() {
    let config = CloudEventConfig {
        force_binary_mode: true,
        ..Default::default()
    };

    let app = Router::new()
        .route("/events", post(handler))
        .with_state(AppState { config });
}
```

### Permissive SpecVersion

```rust
let config = CloudEventConfig {
    strict_specversion: false,
    ..Default::default()
};
```

## Usage with Extensions

### Reading Extension Attributes

```rust,no_run
async fn handler(event: CloudEvent<MyData>) {
    // Access extension by name
    if let Some(traceparent) = event.metadata.extensions.get("traceparent") {
        println!("Trace ID: {}", traceparent);
    }
    
    // Access extension with typed parsing
    if let Some(count) = event.metadata.extensions.get_as::<i32>("count")? {
        if let Ok(n) = count {
            println!("Count: {}", n);
        }
    }
}
```

### Adding Extension Attributes

```rust,no_run
let event = CloudEvent::builder()
    .extension("customext", "custom-value")
    .extension("anotherext", 42)
    .data(payload)
    .build()?;
```

## Test Coverage

The library includes comprehensive test coverage for:

- ✅ Structured mode with data
- ✅ Structured mode without data
- ✅ Missing required field validation
- ✅ Invalid specversion validation
- ✅ Extension attribute handling
- ✅ Binary mode basic parsing
- ✅ Binary mode with optional headers
- ✅ Binary mode with extensions
- ✅ Binary mode empty body
- ✅ Binary mode missing header validation

Run tests:
```bash
cargo test -p axum-cloudevents-core
```

## Examples

- [simple](examples/simple/) - Basic CloudEvent handler
- [tagged-routing](examples/tagged-routing/) - Tagged enum routing
- [binary-mode](examples/binary-mode/) - Binary mode with forced mode
- [response-builder](examples/response-builder/) - Response helpers and derive macros
- [knative-template](examples/knative-template/) - Full Knative service template

## Feature Flags

- `default` - Core functionality
- `tagged` - Tagged enum routing (enabled by default)
- `tracing` - Enable tracing integration

## API Reference

### Core Types

- [`CloudEvent<T>`] - Main extractor for typed events
- [`CloudEventMetadata`] - Event metadata/attributes
- [`CloudEventConfig`] - Configuration for extraction
- [`Extensions`] - Extension attributes
- [`CloudEventBuilder<T>`] - Fluent builder API
- [`IntoCloudEventResponse`] - Trait for auto-response wrapping
- [`CloudEventTagged`] - Trait for enum routing

### Traits

- [`FromRequest<S, B>`] - Implemented by `CloudEvent<T>`
- [`IntoResponse`] - Implemented by `CloudEvent<T>` and `IntoCloudEventResponse`

### Helper Functions

- [`cloud_event()`] - Quick CloudEvent creation helper
- [`format_timestamp_now()` - RFC3339 timestamp generation

## Crate Structure

```
crates/
├── axum-cloudevents-core/      # Core extractor implementation
│   ├── src/
│   │   ├── extractor.rs       # CloudEvent<T> extractor
│   │   ├── metadata.rs        # CloudEventMetadata
│   │   ├── error.rs           # Error types with thiserror
│   │   ├── builder.rs         # CloudEventBuilder
│   │   ├── http.rs            # Binary mode header parsing
│   │   ├── response.rs        # IntoCloudEventResponse trait
│   │   └── tests/
│       ├── binary_mode.rs
│       └── structured_mode.rs
│       └── mod.rs
│
├── axum-cloudevents-macros/    # Proc macros
│   └── src/
│       └── lib.rs             # Derive macros
├── axum-cloudevents-tagged/  # Tagged enum routing
│   └── src/
│   │   └── extractor.rs       # Tagged CloudEvent<T> extractor
├── axum-cloudevents/           # Facade crate with re-exports
│   └── src/
│       └── lib.rs
```

## Development Status

✅ **Compilation**: All crates compile successfully
✅ **Tests**: All 5 tests passing
✅ **Examples**: All 5 examples ready
✅ **Documentation**: Complete with usage examples
✅ **Features**:
- Binary and Structured content modes
- Type-safe event extraction
- Tagged enum routing with catch-all support
- Extension attributes
- Response builder API
- Custom configuration options
- Tracing integration
- Comprehensive error handling

## License

MIT OR Apache-2.0
