---
"repo-roller-frontend": minor
"repo_roller_api": minor
---

The frontend now participates fully in RepoRoller's observability stack, closing the gap left
by Task 1 (structured backend logging) and Task 2 (backend distributed tracing).

**Structured logging**: the frontend's SvelteKit server process now emits structured JSON
lines (`level`, `message`, `time`, plus caller-supplied metadata) to stdout instead of raw
`console.*` output, matching the backend's log shape. `Error` values in metadata are
serialized as `{name, message, stack}` rather than silently collapsing to `{}`, and circular
references or function values embedded in metadata are neutralized instead of crashing the log
call.

**Frontend OpenTelemetry tracing**: the frontend can now export its own OpenTelemetry spans
over OTLP — **opt-in and disabled by default** (fail-safe-off), mirroring the backend's
`OTEL_TRACES_ENABLED` pattern. The SvelteKit server process (Node) exports over OTLP/gRPC via
`OTEL_TRACES_ENABLED` / `OTEL_EXPORTER_OTLP_ENDPOINT` / `OTEL_SERVICE_NAME`; the browser
exports over OTLP/HTTP via the separate, browser-exposed `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT`
(leaving it unset keeps browser export off entirely — there is no separate browser "enabled"
flag). Page load and form submission on the `/create` flow, and every outbound API call, are
instrumented.

**Full trace propagation, browser → SvelteKit server → Rust API**: a standard W3C
`traceparent` header is now attached to every outbound hop — the browser's `apiFetch` calls,
the SvelteKit server's proxied call to the backend, and (via a small backend-side addition)
the Rust API's inbound request handling, which now extracts and continues an inbound
`traceparent` instead of always minting a fresh root span. A browser-initiated
`create_repository` request therefore now produces **one continuous trace** spanning browser
→ SvelteKit server → Rust API → GitHub API calls, viewable end-to-end in the local Jaeger
instance bundled with `examples/local-stack/docker-compose.yml` (now wired with the frontend's
`OTEL_*` / `PUBLIC_OTEL_*` environment variables alongside the Jaeger service added in Task 2).

This is a `minor` bump for `repo-roller-frontend`: it gains new, additive, opt-in-by-default
observability capability (structured logging, OTel tracing, trace propagation) with no change
to any existing page, route, or user-facing behavior when tracing is left disabled — the one
exception, replacing a raw `console.warn` call with the new structured logger, is an
implementation-detail improvement with no change to what operators see in container logs
beyond the output now being a single JSON line instead of a formatted string.

This is a `minor` bump for `repo_roller_api` rather than `patch`: `tracing_middleware` now
honors an inbound `traceparent` header when present, which changes an *observable* outcome —
which trace a request's spans are recorded under — even though no request/response schema,
status code, or default behavior changed for any caller that doesn't send the header (fully
backward compatible, fail-safe: absent or malformed headers fall back to exactly today's
behavior, a fresh root span). The global trace-context propagator this relies on is installed
unconditionally at startup, independent of whether this process's own `OTEL_TRACES_ENABLED`
span export is turned on, so a deployment can continue incoming traces even while exporting
none of its own spans.
