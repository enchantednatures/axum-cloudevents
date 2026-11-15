# Changelog

All notable changes to the `axum-cloudevents` crates are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive rustdoc for the extractor, metadata, error type, header
  constants, and the `CloudEventTagged` derive; both crates enable
  `#![warn(missing_docs)]`.
- `docs/USAGE.md`, `docs/PUBLISHING.md`, `CONTRIBUTING.md`, `SECURITY.md`, and
  `CHANGELOG.md`.
- Runnable `simple` and `tagged` examples.
- Documentation tests for the crate-level and extractor examples.

### Fixed

- The tagged-routing documentation now describes the actual
  `from_cloud_event_parts` router helper instead of implying automatic
  `Deserialize` integration.

## [0.1.1] - 2026-11-14

### Fixed

- Prefer a structured CloudEvent body when a request also carries `ce-*`
  headers. The Knative Kafka broker dispatcher sends both; treating it as binary
  failed to deserialize the envelope into the payload type. Genuine binary
  events are unchanged.

### Added

- Regression tests for structured-body-with-binary-headers and genuine binary
  events.

## [0.1.0] - 2026-11-14

### Added

- `CloudEvent<T>` extractor for Axum 0.8 with structured and binary content
  modes.
- CloudEvents 1.0 metadata, header constants, and accessors.
- `CloudEventTagged` derive for manual event-type routing.
- `CloudEventError` with `IntoResponse` mapping to `400 Bad Request` JSON
  errors.
- Initial CI, workspace, and Kellnr publishing configuration.
