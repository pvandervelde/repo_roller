/**
 * Lightweight, library-agnostic span helpers for instrumenting discrete user
 * operations (form submissions, data loads) from Svelte components and
 * server code.
 *
 * STATUS: STUB — not yet implemented. `withSpan` currently throws. This file
 * defines the CONTRACT the Coder must implement; it is not production code.
 *
 * Contract for the Coder:
 * - Implement `withSpan` using the OpenTelemetry JS tracer appropriate to the
 *   calling context (Web SDK tracer in browser code, Node SDK tracer in
 *   server code).
 * - `withSpan(name, fn, attributes?)` MUST:
 *     1. Start a span named exactly `name`, with `attributes` (if given) set
 *        on it at start.
 *     2. Run `fn`, passing it a `SpanHandle` for the span, with that span set
 *        as the active span for the duration of `fn` (so any nested
 *        `fetch()` calls / child spans attach to it).
 *     3. If `fn` throws or its returned promise rejects: call
 *        `span.recordException(err)`, mark the span as errored, END the
 *        span, and re-throw the ORIGINAL error completely unchanged (same
 *        reference, no wrapping) — callers must see identical
 *        error-handling behaviour with or without instrumentation.
 *     4. If `fn` resolves: end the span and return the resolved value
 *        UNCHANGED.
 *     5. End the span EXACTLY ONCE on every path (success, sync throw, async
 *        rejection) — never zero times, never more than once.
 * - `withSpan` MUST call `fn` EXACTLY ONCE. It must not retry, debounce, or
 *   speculatively pre-invoke `fn`.
 * - `withSpan` MUST work for both synchronous and asynchronous `fn` (its own
 *   return type is always `Promise<T>`).
 *
 * Span name constants are centralised here (rather than inlined as string
 * literals at call sites) so every call site and every test refer to the
 * same source of truth, and so a call-site mix-up (e.g. using the wrong
 * operation's name) is a simple equality check away from being caught.
 */

import { SpanStatusCode, trace, type Exception } from '@opentelemetry/api';

/** Canonical span names for instrumented user-facing operations. */
export const SPAN_NAMES = {
  /** Wraps the `createRepository()` call made when the user submits the /create wizard. */
  CREATE_REPOSITORY_SUBMIT: 'create_repository.submit',
  /** Wraps the `listTemplates()` call made when the /create wizard mounts. */
  LOAD_TEMPLATES: 'create_repository.load_templates',
  /**
   * Wraps the entire `resolve(event)` call inside `hooks.server.ts`'s
   * `handle` hook — represents the SvelteKit-server's own hop in the
   * browser → SvelteKit-server → Rust-backend distributed trace.
   */
  SVELTEKIT_REQUEST: 'sveltekit.request',
} as const;

export interface SpanHandle {
  setAttribute(key: string, value: string | number | boolean): void;
  recordException(err: unknown): void;
}

/** Tracer name shared by every span this module creates. */
const TRACER_NAME = 'repo-roller-frontend';

/**
 * Convert an arbitrary thrown/rejected value into something the OTel API's
 * `recordException()` accepts without itself throwing — `Error` instances
 * and strings are passed through untouched; anything else is coerced to a
 * descriptive string so a non-Error/non-string rejection can never break
 * instrumentation.
 */
function toRecordableException(err: unknown): Exception {
  if (err instanceof Error || typeof err === 'string') return err;
  try {
    return String(err);
  } catch {
    return 'Unserializable exception value';
  }
}

/**
 * Run `fn` inside a named span. See the contract above for exact semantics.
 */
export async function withSpan<T>(
  name: string,
  fn: (span: SpanHandle) => Promise<T> | T,
  attributes?: Record<string, string | number | boolean>,
): Promise<T> {
  const tracer = trace.getTracer(TRACER_NAME);
  return tracer.startActiveSpan(name, async (span) => {
    if (attributes) {
      span.setAttributes(attributes);
    }
    const handle: SpanHandle = {
      setAttribute: (key, value) => span.setAttribute(key, value),
      recordException: (err) => span.recordException(toRecordableException(err)),
    };
    try {
      const result = await fn(handle);
      span.end();
      return result;
    } catch (err) {
      span.recordException(toRecordableException(err));
      span.setStatus({ code: SpanStatusCode.ERROR });
      span.end();
      throw err;
    }
  });
}

/**
 * Run `fn` inside a span representing ONE inbound SvelteKit server request
 * (the SvelteKit-server's own hop in the browser → SvelteKit-server →
 * Rust-backend distributed trace), parented to whatever W3C `traceparent`
 * context can be extracted from `request`'s headers.
 *
 * STATUS: STUB — not yet implemented. Currently always throws. This function
 * defines the CONTRACT the Coder must implement; it is not production code.
 *
 * Intended caller: `src/hooks.server.ts`'s `handle` hook, wrapping its call
 * to `resolve(event)` as `withRequestSpan(event.request, () => resolve(event))`
 * (or equivalent) so that any code running during request handling —
 * including `proxy.server.ts`'s `proxyToBackend()` → `getCurrentTraceparent()`
 * call later in the same request — finds this span as the "current active
 * span" via `trace.getActiveSpan()` and derives a *continuing* `traceparent`
 * from it, rather than minting a disconnected new root trace.
 *
 * Contract for the Coder:
 * - MUST extract an OTel `Context` from `request.headers` using
 *   `@opentelemetry/api`'s `propagation.extract()` against the GLOBALLY
 *   REGISTERED `TextMapPropagator` (e.g. via an `Extractor` adapter over the
 *   standard `Headers` object) — the same conceptual mechanism as the Rust
 *   backend's `extract_trace_context` (see
 *   `crates/repo_roller_api/src/middleware.rs`), ported to the OTel JS API.
 *   Must NOT hand-parse the `traceparent` string itself — that would
 *   duplicate `trace-context.ts`'s existing, independently-tested parsing
 *   logic and risks the two drifting out of sync.
 * - A globally registered `TextMapPropagator` capable of decoding W3C
 *   `traceparent` headers is a PRECONDITION for this function to ever see a
 *   non-empty extracted context — mirroring the Rust precedent's own
 *   "Precondition (production wiring)" note on `extract_trace_context`.
 *   CONFIRMED ALREADY SATISFIED: `observability.server.ts`'s
 *   `initServerObservability()` calls `NodeTracerProvider#register()` with
 *   no arguments, and `@opentelemetry/sdk-trace-node`'s `register()`
 *   defaults (when `config.propagator`/`config.contextManager` are
 *   `undefined`, as they are here) to installing a `CompositePropagator` of
 *   `W3CTraceContextPropagator` + `W3CBaggagePropagator` AND an
 *   `AsyncLocalStorageContextManager` globally — so both W3C extraction and
 *   correct active-context propagation across `await` boundaries already
 *   work out of the box with NO further SDK registration changes needed.
 *   Nonetheless, if some future refactor ever passes `propagator: null`
 *   there (or `initServerObservability()` has not yet run for any reason),
 *   extraction harmlessly yields an empty context — this function MUST fall
 *   back to a fresh root span in that case rather than throwing or hanging.
 * - If the header is absent, syntactically malformed, or decodes to an
 *   invalid (e.g. all-zero trace-id/span-id) `SpanContext`, the extracted
 *   context carries no valid span; this is harmless and MUST fall back to a
 *   fresh ROOT span (today's pre-fix behaviour) — MUST NOT throw regardless
 *   of header content, and MUST NOT hang/block.
 * - MUST start a new span named `SPAN_NAMES.SVELTEKIT_REQUEST` as a CHILD of
 *   the extracted context (e.g. via
 *   `tracer.startActiveSpan(name, options, extractedContext, callback)`),
 *   and run `fn` with that span set as the active span for fn's ENTIRE
 *   duration, including through every `await` inside `fn` — not just
 *   synchronously before the first `await`.
 * - Same span-lifecycle rules as `withSpan` above: end the span EXACTLY ONCE
 *   on every path (success, sync throw, async rejection) — never zero times,
 *   never more than once; on failure, call `span.recordException`, mark
 *   `SpanStatusCode.ERROR`, end the span, and re-throw the ORIGINAL error
 *   reference completely unchanged (no wrapping); on success, end the span
 *   and resolve with `fn`'s returned/awaited value UNCHANGED.
 * - MUST call `fn` EXACTLY ONCE. MUST NOT retry, debounce, or speculatively
 *   pre-invoke `fn`.
 * - MUST NOT mutate `request` or its headers.
 * - MUST NOT leak the active span context beyond `fn`'s execution — once the
 *   returned promise has settled (success or failure), `trace.getActiveSpan()`
 *   observed from OUTSIDE this call must no longer see this request's span
 *   as active.
 * - Depends on `request` only through the standard `Request.headers`
 *   interface — no coupling to SvelteKit's `RequestEvent` shape, so this
 *   stays testable (and reusable) independent of SvelteKit's routing
 *   machinery.
 */
export async function withRequestSpan<T>(
  request: Request,
  fn: (span: SpanHandle) => Promise<T> | T,
): Promise<T> {
  throw new Error(
    `spans.ts: withRequestSpan() not implemented (request url=${request.url}, fn=${typeof fn})`,
  );
}
