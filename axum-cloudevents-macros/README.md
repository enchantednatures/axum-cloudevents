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
fully prefixed type or the bare suffix.

> Not affiliated with or endorsed by the Axum project.

## License

MIT
