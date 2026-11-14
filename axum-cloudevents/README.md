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

See the [repository README](https://github.com/enchantednatures/axum-cloudevents)
for the registry configuration, quick start, tagged routing examples, and
feature flags.

> Not affiliated with or endorsed by the Axum project.

## License

MIT
