# Publishing

Both crates are published to the private Kellnr registry. The GitHub Actions
workflow in `.github/workflows/publish.yaml` handles normal releases.

## Credentials

The workflow uses the `KELLNR_PUBLISH_TOKEN` repository secret. The token must
belong to a user with publish rights. The read-only `KELLNR_TOKEN` secret used
by consumers and CI cannot publish.

For a local publish, authenticate with `cargo login --registry kellnr` or set
`CARGO_REGISTRIES_KELLNR_TOKEN`. The token is not stored in the repository.

## Versioning

Versions are centralized in `[workspace.package]` in the root `Cargo.toml`.
Both crates share the same version. The main crate depends on the macros crate
through `[workspace.dependencies]` with a `0.1` range, so a patch release does
not require a dependency edit.

Release checklist:

1. Update `version` in `[workspace.package]`.
2. Add a `CHANGELOG.md` entry.
3. Commit and push the change.
4. Create and push a tag named `crates-vX.Y.Z`:

   ```bash
   git tag crates-v0.2.0
   git push origin crates-v0.2.0
   ```

5. The workflow publishes `axum-cloudevents-macros` first, then
   `axum-cloudevents`.
6. If the consuming template needs a new minor range, update its
   `Cargo.toml.liquid` dependency.

Do not reuse a version. Kellnr rejects duplicate versions and versions that
differ only in build metadata.

## Workflow behavior

On a `crates-v*` tag the workflow:

1. Verifies the tag version matches both crate versions.
2. Runs formatting, Clippy, and tests.
3. Publishes `axum-cloudevents-macros`.
4. Waits briefly for the index, then publishes `axum-cloudevents`.
5. Skips versions that already exist, so re-running a tag is safe.

The manual `workflow_dispatch` trigger supports a `dry_run` input for packaging
and verification without uploading.

## Manual publish

From a clean working tree:

```bash
cargo publish -p axum-cloudevents-macros --registry kellnr
cargo publish -p axum-cloudevents --registry kellnr
```

`cargo publish` ignores the workspace `[patch.kellnr]` entry, so the published
manifest points at the registry.

## Consuming a release

```toml
[dependencies]
axum-cloudevents = { version = "0.1", registry = "kellnr" }
```

The consumer needs the `kellnr` registry configured and a read token:

```toml
[registries.kellnr]
index = "sparse+https://crates.enchantednatures.com/api/v1/crates/"
credential-provider = ["cargo:token"]
```

See the template's `docs/CRATES.md` for the full consumer setup, including
Docker BuildKit secrets and CI.

## Token management

- `KELLNR_TOKEN` — read-only user token for CI and generated projects.
- `KELLNR_PUBLISH_TOKEN` — publisher token for this repository.

Tokens are created through the Kellnr API or UI. The token value is shown once;
store it immediately in the repository secret:

```bash
printf '%s' "$TOKEN" | gh secret set KELLNR_PUBLISH_TOKEN \
  --repo enchantednatures/axum-cloudevents
```

To rotate, create a new token, update the secret, then delete the old token.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `registry kellnr requires authentication` | No token available | Set `CARGO_REGISTRIES_KELLNR_TOKEN` or run `cargo login --registry kellnr` |
| `403` on publish | Token belongs to a read-only user | Use a publisher token |
| `500` with `507 Insufficient Storage` in Kellnr logs | Kellnr's S3 backend is full | Free space or move the Kellnr buckets |
| `crate version already exists` | Version was already published | Bump the version; do not reuse it |
| Main crate cannot resolve the macros crate | Macros publish not visible yet | Wait for the index and retry; the workflow retries after a short delay |
