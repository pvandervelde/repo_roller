/**
 * Browser-side OpenTelemetry SDK bootstrap.
 *
 * STATUS: STUB — not yet implemented. `initClientObservability` and
 * `isClientObservabilityInitialized` currently throw. This file defines the
 * CONTRACT the Coder must implement; it is not production code.
 *
 * This module is browser-only. It MUST NOT be imported from server-side
 * code (`*.server.ts`), and it MUST fail fast (throw a clear, descriptive
 * error) if it is ever invoked in a context where `window` is undefined,
 * rather than silently doing nothing — a silent no-op would hide the bug of
 * having been mis-imported into SSR code.
 *
 * Contract for the Coder:
 * - `initClientObservability()` registers a browser trace provider with
 *   fetch instrumentation (so `apiFetch()`'s calls to the SvelteKit server
 *   are automatically captured as child spans of whatever page-level span is
 *   active, e.g. a page-load or form-submission span) exactly once per page
 *   session.
 * - Calling `initClientObservability()` more than once MUST be a safe no-op
 *   the second and subsequent times — no duplicate provider registration, no
 *   thrown error.
 * - MUST throw synchronously, BEFORE doing any OTel registration work, if
 *   `typeof window === 'undefined'` — this module is browser-only.
 * - `isClientObservabilityInitialized()` reports whether init has
 *   successfully completed for this page session.
 */

/**
 * Initialise the browser OTel SDK for this page session. Safe to call
 * multiple times; only the first call has an effect. Throws if called
 * outside a browser context (`window` undefined).
 *
 * STUB: throws "not implemented" until implemented — this is a deliberately
 * wrong message so it is distinguishable from the eventual real
 * browser-context guard-clause error.
 */
export function initClientObservability(): void {
  throw new Error('observability.client.ts: initClientObservability() not implemented');
}

/**
 * Whether `initClientObservability()` has successfully run in this page
 * session.
 *
 * STUB: throws until implemented.
 */
export function isClientObservabilityInitialized(): boolean {
  throw new Error('observability.client.ts: isClientObservabilityInitialized() not implemented');
}

/**
 * Test-only hook: reset module-level init state between tests.
 */
export function _resetObservabilityForTesting(): void {
  // Intentionally a no-op for now: there is no state to reset yet because
  // the real implementation (and its module-level init flag) does not exist
  // yet. The Coder must make this actually reset that flag once it exists.
}
