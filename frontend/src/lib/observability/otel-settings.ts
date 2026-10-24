/**
 * Pure OTLP export configuration resolution.
 *
 * Deliberately mirrors `crates/repo_roller_api/src/tracing_otel.rs`'s
 * `OtelSettings`/`resolve_otel_settings` in behaviour (same env var names,
 * same defaults, same fail-safe-off semantics) so the frontend and backend
 * agree on what "OTLP export is enabled" means operationally — even though
 * each process resolves its OWN instance of these settings from its OWN
 * environment.
 *
 * ## Contract
 * - Fail-safe default is `enabled: false`. Only the case-insensitive
 *   literal values `"true"` or `"1"` enable export; every other value
 *   (unset/`null`/`undefined`, empty string, `"false"`, `"0"`, garbage,
 *   whitespace-only) resolves to `enabled: false`. An unset or
 *   misconfigured env var must NEVER silently turn on background network
 *   export to an address nobody asked for.
 * - `otlpEndpoint` defaults to `DEFAULT_OTLP_ENDPOINT` when the endpoint
 *   input is unset/`null`/`undefined`/empty; otherwise the raw value
 *   verbatim (no validation here — malformed-endpoint handling is the
 *   exporter constructor's job, mirroring the Rust precedent's
 *   `build_otlp_span_exporter`).
 * - `serviceName` defaults to `DEFAULT_SERVICE_NAME` when the service-name
 *   input is unset/`null`/`undefined`/empty; otherwise the raw value
 *   verbatim.
 * - MUST be a pure function: never throws regardless of input, never
 *   performs I/O, two calls with the same inputs always produce
 *   deep-equal output, and calls must not leak state across each other.
 *
 * ## Server vs. browser: two different endpoints, on purpose
 * This module resolves settings from NODE-SIDE environment variable values
 * (read by the caller — `observability.server.ts` — from `process.env`,
 * directly, the same way `src/lib/session.server.ts` reads
 * `process.env.SESSION_SECRET`; deliberately NOT via `$env/dynamic/private`,
 * which snapshots its value at module-resolution time under Vite's dev/test
 * "serve" command and is awkward to control from a test — see
 * `tests/proxy-trace-propagation.test.ts`'s header comment for the fuller
 * explanation of that gotcha discovered while testing `proxy.server.ts`).
 *
 * The BROWSER cannot read `process.env` at all. Its OTLP endpoint is a
 * SEPARATE concern, deliberately a different env var
 * (`PUBLIC_OTEL_ENDPOINT_ENV_VAR` below) with a different runtime value in
 * practice: the Node-side endpoint is typically a Docker-network-internal
 * gRPC address (e.g. `http://jaeger:4317`, unreachable from the end user's
 * browser), while the browser needs a publicly-routable OTLP/HTTP address
 * (e.g. `http://localhost:4318/v1/traces` for local dev, or a real public
 * collector hostname in production) — gRPC is not usable from a browser at
 * all. See `observability.client.ts`'s `ClientObservabilityOptions` for how
 * that value is expected to reach `initClientObservability()`.
 */

export const OTEL_ENABLED_ENV_VAR = 'OTEL_TRACES_ENABLED';
export const OTEL_ENDPOINT_ENV_VAR = 'OTEL_EXPORTER_OTLP_ENDPOINT';
export const OTEL_SERVICE_NAME_ENV_VAR = 'OTEL_SERVICE_NAME';

/**
 * Name of the SvelteKit PUBLIC (browser-exposed) env var carrying the OTLP
 * endpoint the BROWSER should export spans to.
 *
 * MUST be read via `$app/env/public` — declared in `src/env.ts` with
 * `public: true` and no `static: true` — by the caller that wires this into
 * `initClientObservability()` (expected to be `src/hooks.client.ts`, updated
 * by the Coder). Rationale: this app's entire configuration model
 * (`BACKEND_API_URL`, `GITHUB_ORG`, `ORIGIN`, `SESSION_SECRET`,
 * `brand.toml`) is "build the Docker image once, configure every value via
 * `docker-compose.yml`'s `environment:` block at container START" (see
 * `examples/local-stack/docker-compose.yml`) — i.e. RUNTIME configuration,
 * not build-time. A `static: true` declaration would be inlined by Vite at
 * `vite build` time, which happens INSIDE the Dockerfile, before any
 * `docker compose up` env vars exist — using it here would freeze this
 * value at whatever it was during the image build (almost certainly
 * unset). Leaving `static` at its default (`false`, i.e. dynamic) instead
 * reads the value at actual server-process runtime for a production
 * (`adapter-node`) build, exactly like `BACKEND_API_URL` already is in
 * `proxy.server.ts`.
 *
 * Named with a `PUBLIC_` prefix purely by convention, matching the old
 * `$env/*\/public` naming — exposure to client bundles is actually
 * controlled by the explicit `public: true` field in `src/env.ts`, not by
 * the variable name, which is exactly why the browser-facing name cannot
 * simply reuse `OTEL_ENDPOINT_ENV_VAR` (that one defaults to private).
 */
export const PUBLIC_OTEL_ENDPOINT_ENV_VAR = 'PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT';

/**
 * Default OTLP collector endpoint used when `OTEL_ENDPOINT_ENV_VAR` is unset
 * or empty. Matches `examples/local-stack/docker-compose.yml`'s Jaeger OTLP
 * gRPC receiver port (4317) — the same transport/port the Rust backend's
 * `tracing_otel::DEFAULT_OTLP_ENDPOINT` targets, for symmetry.
 */
export const DEFAULT_OTLP_ENDPOINT = 'http://localhost:4317';

/** Default service name used when `OTEL_SERVICE_NAME_ENV_VAR` is unset or empty. */
export const DEFAULT_SERVICE_NAME = 'repo-roller-frontend';

/** Resolved OpenTelemetry/OTLP export configuration for a process. */
export interface OtelSettings {
  /** Whether OTel span export should be attached at all. Fail-safe default is `false`. */
  enabled: boolean;
  /** OTLP collector endpoint (transport depends on which exporter the caller constructs). */
  otlpEndpoint: string;
  /** Service name attached to every exported span's resource attributes. */
  serviceName: string;
}

/**
 * Resolves {@link OtelSettings} from the raw string values of the relevant
 * environment variables, or `null`/`undefined` if a given one is unset. See
 * the module-level contract for the exact matching/default rules. Never
 * throws.
 */
export function resolveOtelSettings(
  enabledRaw: string | null | undefined,
  endpointRaw: string | null | undefined,
  serviceNameRaw: string | null | undefined,
): OtelSettings {
  const enabled = enabledRaw != null && (enabledRaw.toLowerCase() === 'true' || enabledRaw === '1');

  const otlpEndpoint =
    endpointRaw != null && endpointRaw !== '' ? endpointRaw : DEFAULT_OTLP_ENDPOINT;

  const serviceName =
    serviceNameRaw != null && serviceNameRaw !== '' ? serviceNameRaw : DEFAULT_SERVICE_NAME;

  return { enabled, otlpEndpoint, serviceName };
}
