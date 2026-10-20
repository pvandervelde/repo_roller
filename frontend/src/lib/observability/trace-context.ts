/**
 * W3C Trace Context (`traceparent` header) helpers — pure, framework-agnostic.
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

import { trace } from '@opentelemetry/api';

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

const ALL_ZERO_TRACE_ID = '0'.repeat(32);
const ALL_ZERO_SPAN_ID = '0'.repeat(16);
/** Bit 0 of the W3C trace-flags byte marks a trace as sampled. */
const SAMPLED_FLAG_BIT = 1;

/**
 * Fill `bytes` with cryptographically-strong random values when available,
 * falling back to `Math.random()` (e.g. in an environment with no `crypto`
 * global) so id minting can never throw.
 */
function fillRandomBytes(bytes: Uint8Array<ArrayBuffer>): Uint8Array<ArrayBuffer> {
  const cryptoObj = (globalThis as { crypto?: Crypto }).crypto;
  if (cryptoObj?.getRandomValues) {
    return cryptoObj.getRandomValues(bytes);
  }
  for (let i = 0; i < bytes.length; i += 1) {
    bytes[i] = Math.floor(Math.random() * 256);
  }
  return bytes;
}

/** Mint a random, non-all-zero lowercase hex id of `byteLength` bytes. */
function mintHexId(byteLength: number): string {
  const zeroPattern = new RegExp(`^0{${byteLength * 2}}$`);
  let hex: string;
  do {
    const bytes = fillRandomBytes(new Uint8Array(byteLength));
    hex = Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('');
  } while (zeroPattern.test(hex));
  return hex;
}

/** Mint a fresh, non-all-zero 32-hex-char (16 byte) trace id. */
function mintTraceId(): string {
  return mintHexId(16);
}

/** Mint a fresh, non-all-zero 16-hex-char (8 byte) span id. */
function mintSpanId(): string {
  return mintHexId(8);
}

/**
 * Return the span context of the currently active span (e.g. via the OTel JS
 * API's `trace.getActiveSpan()`), or `null` if there is no active span in
 * scope for the calling context.
 *
 * Also returns `null` if the active span's context is the OTel "invalid"
 * sentinel (all-zero trace-id/span-id) that the API's no-op tracer produces
 * when no real SDK has been registered — that sentinel is never a valid
 * `traceparent` value and must be treated the same as "no active span".
 */
export function getActiveSpanContext(): ActiveSpanContext | null {
  try {
    const span = trace.getActiveSpan();
    if (!span) return null;

    const spanContext = span.spanContext();
    if (!spanContext) return null;
    if (spanContext.traceId === ALL_ZERO_TRACE_ID || spanContext.spanId === ALL_ZERO_SPAN_ID) {
      return null;
    }

    return {
      traceId: spanContext.traceId,
      spanId: spanContext.spanId,
      sampled: (spanContext.traceFlags & SAMPLED_FLAG_BIT) === SAMPLED_FLAG_BIT,
    };
  } catch {
    return null;
  }
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
 */
export function buildTraceparentHeader(parent?: ActiveSpanContext | null): string {
  const traceId = parent?.traceId ?? mintTraceId();
  const spanId = parent?.spanId ?? mintSpanId();
  // When minting a fresh root trace there is no prior sampling decision to
  // honour, so default to "sampled" — a freshly-minted trace is exactly the
  // one we want recorded downstream.
  const sampled = parent?.sampled ?? true;
  const flags = sampled ? '01' : '00';
  return `00-${traceId}-${spanId}-${flags}`;
}

/**
 * Convenience wrapper used directly by `apiFetch()` and `proxyToBackend()`:
 * looks up the active span via `getActiveSpanContext()` and formats it (or
 * mints a fresh root trace if there is none) via `buildTraceparentHeader()`.
 *
 * MUST NOT throw under any circumstance — logging/tracing must never be able
 * to break a real HTTP request. MUST always return a string matching
 * `TRACEPARENT_FORMAT`.
 */
export function getCurrentTraceparent(): string {
  try {
    return buildTraceparentHeader(getActiveSpanContext());
  } catch {
    // getActiveSpanContext() already guards against throwing, but
    // buildTraceparentHeader() itself could theoretically throw on a future
    // change; a tracing bug must never be able to break a real HTTP request.
    return buildTraceparentHeader(null);
  }
}
