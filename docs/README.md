# axum-cloudevents documentation

| Guide | Contents |
| --- | --- |
| [Usage](USAGE.md) | Installation, content modes, metadata, tagged routing, errors, testing, examples |
| [Publishing](PUBLISHING.md) | Kellnr releases, versioning, tokens, troubleshooting |
| [Changelog](../CHANGELOG.md) | Release history |
| [Contributing](../CONTRIBUTING.md) | Development setup, checks, commit conventions |
| [Security](../SECURITY.md) | Supported versions and vulnerability reporting |

## Source

<https://github.com/enchantednatures/axum-cloudevents>

## Registry

The crates are published to the private Kellnr registry:

```toml
[registries.kellnr]
index = "sparse+https://crates.enchantednatures.com/api/v1/crates/"
credential-provider = ["cargo:token"]
```

```toml
[dependencies]
axum-cloudevents = { version = "0.1", registry = "kellnr" }
```
