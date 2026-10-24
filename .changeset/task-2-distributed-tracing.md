---
"repo_roller_api": minor
"repo_roller_core": minor
"github_client": minor
"config_manager": patch
---

Backend API can now export OpenTelemetry traces over OTLP for the full repository-creation
flow. This is **opt-in and disabled by default** (fail-safe-off) — set
`OTEL_TRACES_ENABLED=true` to turn it on, and configure `OTEL_EXPORTER_OTLP_ENDPOINT` /
`OTEL_SERVICE_NAME` to control where spans are sent and how the service identifies itself.
When enabled, a `create_repository` request produces one trace with child spans for each
phase boundary (authentication, configuration resolution, template processing, repository
creation) plus every underlying GitHub API call as a properly nested child span — nothing is
orphaned or left untraced. `OTEL_EXPORTER_OTLP_ENDPOINT` supports both `http://` (local
development) and `https://` (encrypted, required for any collector reachable outside a
private network) endpoints. Local development gets a working trace viewer out of the box:
`examples/local-stack/docker-compose.yml` now bundles a Jaeger all-in-one service with its
UI at `http://localhost:16686`. Existing `tracing::info!`/`warn!`/`error!` call sites and
JSON log output are unaffected — this is purely additive, composed alongside the existing
sanitizing JSON logging layer.

Every business-context value attached to a span (organization, template, requestor, and
equivalents recorded elsewhere in the GitHub API client) is sanitized before export: values
that look like tokens, `Bearer` headers, JWTs, or high-entropy secrets are replaced with
`[REDACTED]` in their entirety, and all other values are length-bounded. This is a separate
mechanism from — but provides the same guarantee as — the log-line redaction shipped
previously: neither log output nor exported trace data ever carries a raw secret or an
unbounded, attacker-controlled string.

This is a `minor` bump for `repo_roller_api`, `repo_roller_core`, and `github_client`: each
gains new, additive, opt-in observability capability (a new tracing bridge module, a new
public span-hierarchy/business-context-attribute module, and a new public span-attribute
sanitization module respectively) with no change to any existing public API or default
runtime behavior when tracing is left disabled.

`config_manager` gets a `patch`: the one instrumentation site in this crate that attaches
business-context fields to a span now routes its values through the same sanitization
function as `github_client`, with no change to any public API or observable behavior for
consumers of the crate.
