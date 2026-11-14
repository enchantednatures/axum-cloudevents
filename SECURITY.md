# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 0.1.x | Yes |
| < 0.1 | No |

Security fixes are released on the latest 0.1.x version.

## Reporting a vulnerability

Please use GitHub's private vulnerability reporting:

<https://github.com/enchantednatures/axum-cloudevents/security/advisories/new>

Include:

- A description of the issue and its impact
- The affected version
- A minimal reproduction or proof of concept
- Any suggested mitigation

Do **not** open a public issue for a security vulnerability.

## Scope

In scope:

- The `axum-cloudevents` extractor and `axum-cloudevents-macros` crate
- Parsing and validation of CloudEvents structured and binary requests
- The `CloudEventTagged` derive expansion
- Documentation and examples that could lead to insecure usage

Out of scope:

- Vulnerabilities in Axum, Tokio, Serde, or Kellnr
- Vulnerabilities in an application that uses these crates
- Resource exhaustion that is addressable with Axum's `DefaultBodyLimit` or
  deployment-level limits
- Findings from automated scanners without a demonstrated impact

## Response

We aim to acknowledge a report within 3 business days and to provide a
remediation plan within 30 days for high or critical issues. We will coordinate
disclosure and credit reporters unless they ask to remain anonymous.
