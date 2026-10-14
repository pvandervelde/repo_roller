/**
 * W3C Trace Context (`traceparent` header) helpers — pure, framework-agnostic.
 *
 * STATUS: STUB — not yet implemented. Every exported function currently
 * throws. This file defines the CONTRACT the Coder must implement; it is not
 * production code.
 *
 * Used by:
 *  - `src/lib/api/client.ts`'s `apiFetch()` (browser → SvelteKit server hop)
 *  - `src/lib/proxy.server.ts`'s `proxyToBackend()` (SvelteKit server → Rust
 *    backend hop)
 *
 * Both of those call sites MUST attach a `traceparent` header to every
 * outgoing request so that, together with the backend's (already-landed)
 * inbound `traceparent` extraction in `repo_roller_api`'s tracing
 * middleware, a single user action's browser → SvelteKit-server → Rust hop
 * chain is recorded as ONE continuous distributed trace.
 *
 * ---------------------------------------------------------------------------
 * Design decision (Tester, RED phase) — why MANUAL propagation, not
 * auto-instrumentation:
 *
 * OTel JS auto-instrumentation (`@opentelemetry/instrumentation-fetch` in the
 * browser, `-http`/`-undici` in Node) works by monkey-patching the global
 * `fetch`/`http`/`undici` transport at SDK-registration time. That makes it
 * fundamentally invisible to — and bypassed by — any unit test that replaces
 * `global.fetch` with a `vi.fn()` (which is exactly what this codebase's
 * existing `tests/api.test.ts` already does, and what new tests in this
 * suite do too). A test tier that cannot observe a behaviour cannot prove it
 * exists, so auto-instrumentation cannot be the mechanism verified here.
 *
 * The task brief explicitly sanctions a fallback: "If auto-instrumentation
 * proves unworkable in this stack, fall back to manual propagation
 * (construct the `traceparent` header value from the active OTel span
 * context, attach it explicitly to `apiFetch`'s `fetch()` calls and to
 * `proxyToBackend`'s outbound `fetch()` call)". This module IS that manual
 * fallback, chosen specifically because it is deterministically testable at
 * this tier. The Coder MAY additionally register OTel auto-instrumentation
 * for other purposes (e.g. automatic span creation for metrics/diagnostics),
 * but the two call sites listed above MUST explicitly call
 * `getCurrentTraceparent()` from this module and set the result as the
 * `traceparent` header themselves — they must not rely solely on
 * auto-instrumentation to do it, because that cannot be verified here and
 * the acceptance criteria requires it to be provably present.
 * ---------------------------------------------------------------------------
 *
 * Format (W3C Trace Context, version "00"):
 *   "00-<32 lowercase hex trace-id>-<16 lowercase hex parent-id>-<2 hex flags>"
 *
 * Per the W3C spec, a trace-id or parent-id that is all-zero is explicitly
 * invalid and MUST never be emitted by a conforming implementation.
 */

/** A minimal, library-agnostic view of an active span's identifying context. */
export interface ActiveSpanContext {
  /** 32 lowercase hex characters. Must not be all zeros. */
  traceId: string;
  /** 16 lowercase hex characters. Must not be all zeros. */
  spanId: string;
  /** Whether this trace should be recorded/sampled. */
  sampled: boolean;
}

/** Regex matching a syntactically valid W3C `traceparent` header value. */
export const TRACEPARENT_FORMAT = /^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$/;

/**
 * Return the span context of the currently active span (e.g. via the OTel JS
 * API's `trace.getActiveSpan()`), or `null` if there is no active span in
 * scope for the calling context.
 *
 * STUB: throws until implemented.
 */
export function getActiveSpanContext(): ActiveSpanContext | null {
  throw new Error('trace-context.ts: getActiveSpanContext() not implemented');
}

/**
 * Build a well-formed W3C `traceparent` header value.
 *
 * - If `parent` is provided, the returned header MUST reuse that EXACT
 *   trace-id and span-id (continuing the existing trace as a reference to
 *   that span) — it must not mint new identifiers when a parent is given.
 * - If `parent` is `null`/`undefined`, this mints a brand-new root trace
 *   (freshly generated, non-zero trace-id and span-id). Each call with no
 *   parent MUST produce different identifiers than the previous call — this
 *   is the "no active span" fallback described in the acceptance criteria:
 *   the header is always present and well-formed, never omitted.
 *
 * MUST NOT throw. MUST always return a string matching `TRACEPARENT_FORMAT`.
 *
 * STUB: throws until implemented.
 */
export function buildTraceparentHeader(parent?: ActiveSpanContext | null): string {
  throw new Error(
    `trace-context.ts: buildTraceparentHeader() not implemented (parent=${JSON.stringify(parent)})`,
  );
}

/**
 * Convenience wrapper used directly by `apiFetch()` and `proxyToBackend()`:
 * looks up the active span via `getActiveSpanContext()` and formats it (or
 * mints a fresh root trace if there is none) via `buildTraceparentHeader()`.
 *
 * MUST NOT throw under any circumstance — logging/tracing must never be able
 * to break a real HTTP request. MUST always return a string matching
 * `TRACEPARENT_FORMAT`.
 *
 * STUB: throws until implemented.
 */
export function getCurrentTraceparent(): string {
  throw new Error('trace-context.ts: getCurrentTraceparent() not implemented');
}
