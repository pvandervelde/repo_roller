/**
 * Node-side OpenTelemetry SDK bootstrap for the SvelteKit server runtime.
 *
 * STATUS: STUB — not yet implemented. `initServerObservability` and
 * `isServerObservabilityInitialized` currently throw. This file defines the
 * CONTRACT the Coder must implement; it is not production code.
 *
 * MUST NOT be imported from client-side code (server-only).
 *
 * Contract for the Coder:
 * - `initServerObservability()` registers the Node OTel SDK (a trace
 *   provider, plus instrumentation for outbound HTTP/undici requests so that
 *   `proxy.server.ts`'s outbound fetch to the Rust backend is automatically
 *   captured as a child span) exactly once per process.
 * - Calling `initServerObservability()` more than once MUST be a safe no-op
 *   the second and subsequent times — no duplicate provider registration, no
 *   duplicate instrumentation, no thrown error. (OTel's own SDK registration
 *   APIs throw or warn on double-registration in several versions; this
 *   module must guard against that regardless of which underlying API is
 *   used.)
 * - MUST NOT throw if OTLP exporter configuration (e.g.
 *   `OTEL_EXPORTER_OTLP_ENDPOINT`) is unset at call time — fall back to a
 *   safe default (e.g. a no-op/console exporter) rather than crashing server
 *   startup when observability config is incomplete.
 * - `isServerObservabilityInitialized()` reports whether init has
 *   successfully completed in this process.
 */

/**
 * Initialise the Node OTel SDK for this process. Safe to call multiple
 * times; only the first call has an effect.
 *
 * STUB: throws until implemented.
 */
export function initServerObservability(): void {
  throw new Error('observability.server.ts: initServerObservability() not implemented');
}

/**
 * Whether `initServerObservability()` has successfully run in this process.
 *
 * STUB: throws until implemented.
 */
export function isServerObservabilityInitialized(): boolean {
  throw new Error('observability.server.ts: isServerObservabilityInitialized() not implemented');
}

/**
 * Test-only hook: reset module-level init state between tests so each test
 * can observe `initServerObservability()`'s first-call behaviour freshly.
 * Has no effect on any already-registered global OTel provider beyond this
 * module's own bookkeeping flag — real implementations should document any
 * additional teardown needed here once written.
 */
export function _resetObservabilityForTesting(): void {
  // Intentionally a no-op for now: there is no state to reset yet because
  // the real implementation (and its module-level init flag) does not exist
  // yet. The Coder must make this actually reset that flag once it exists.
}
