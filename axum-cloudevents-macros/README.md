# axum-cloudevents-macros

Procedural macros for [`axum-cloudevents`](https://github.com/enchantednatures/axum-cloudevents).

The `CloudEventTagged` derive maps a CloudEvents `type` to an enum variant. It
is normally used through the re-export in `axum-cloudevents`:

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
```

The generated `from_cloud_event_parts(event_type, data)` accepts either the
fully prefixed type or the bare suffix. The derive does **not** implement
`Deserialize` and does not hook into the extractor automatically; extract a
`CloudEvent<serde_json::Value>` and call the method with the event type and
data.

See the [usage guide](https://github.com/enchantednatures/axum-cloudevents/blob/main/docs/USAGE.md)
for the complete routing example.

> Not affiliated with or endorsed by the Axum project.

## License

MIT
