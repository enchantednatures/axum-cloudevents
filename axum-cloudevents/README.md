# axum-cloudevents

CloudEvents 1.0 extractors for Axum 0.8+.

This crate provides `CloudEvent<T>` as an Axum `FromRequest` extractor, plus
metadata accessors and CloudEvents header constants. The `CloudEventTagged`
derive is re-exported from `axum-cloudevents-macros` behind the default
`macros` feature.

```toml
[dependencies]
axum-cloudevents = { version = "0.1", registry = "kellnr" }
```

The `CloudEventTagged` derive generates a manual
`from_cloud_event_parts(event_type, data)` router helper; it does not implement
`Deserialize` or hook into the extractor automatically.

See the [repository README](https://github.com/enchantednatures/axum-cloudevents)
and the [usage guide](https://github.com/enchantednatures/axum-cloudevents/blob/main/docs/USAGE.md)
for the registry configuration, content modes, tagged routing, error handling,
testing, and feature flags.

> Not affiliated with or endorsed by the Axum project.

## License

MIT
