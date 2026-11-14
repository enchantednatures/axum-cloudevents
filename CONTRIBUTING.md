# Contributing

Thanks for taking the time to contribute.

## Prerequisites

- Rust 1.99 (pinned in `rust-toolchain.toml`)
- A Kellnr read token if you want to resolve the published crates without the
  local patch: `cargo login --registry kellnr`
- `gh` for releases (optional)

## Setup

```bash
git clone git@github.com:enchantednatures/axum-cloudevents.git
cd axum-cloudevents

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

The workspace `.cargo/config.toml` contains:

```toml
[patch.kellnr]
axum-cloudevents-macros = { path = "axum-cloudevents-macros" }
```

This makes local development and CI use the workspace macros crate instead of
the published one. `cargo publish` ignores patches, so releases still resolve
the macros crate from Kellnr.

## Layout

```
axum-cloudevents/          # extractor, metadata, headers, re-exported derive
axum-cloudevents-macros/   # CloudEventTagged derive
docs/                      # usage, publishing, and consumer guides
examples/                  # runnable examples
```

## Code guidelines

- Keep public APIs documented. Both crates enable `#![warn(missing_docs)]`.
- Add tests for behavior changes; the extractor's content-mode detection has
  regression tests for structured bodies with binary headers.
- Run `cargo fmt` and Clippy with `-D warnings` before pushing.
- Update `CHANGELOG.md` for user-visible changes.
- Do not add a runtime dependency without a clear need; the extractor is
  intended to stay small.

## Commit messages

Use Conventional Commits:

```
<type>(<scope>): <subject>

Types: feat, fix, docs, style, refactor, test, chore
```

Examples:

```
fix(extractor): prefer structured bodies with binary headers
docs(usage): document tagged routing
test(metadata): cover missing ce-type
```

## Pull requests

Before opening a pull request:

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo doc --workspace --no-deps`
- [ ] Public items documented
- [ ] `CHANGELOG.md` updated when behavior changes

## Releases

See [PUBLISHING.md](docs/PUBLISHING.md). Do not publish from a dirty working
tree or with the read-only token.

## Security

Do not report vulnerabilities in a public issue. See [SECURITY.md](SECURITY.md).
