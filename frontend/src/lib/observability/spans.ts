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

/** Canonical span names for instrumented user-facing operations. */
export const SPAN_NAMES = {
  /** Wraps the `createRepository()` call made when the user submits the /create wizard. */
  CREATE_REPOSITORY_SUBMIT: 'create_repository.submit',
  /** Wraps the `listTemplates()` call made when the /create wizard mounts. */
  LOAD_TEMPLATES: 'create_repository.load_templates',
} as const;

export interface SpanHandle {
  setAttribute(key: string, value: string | number | boolean): void;
  recordException(err: unknown): void;
}

/**
 * Run `fn` inside a named span. See the contract above for exact semantics.
 *
 * STUB: throws synchronously (before even calling `fn`) until implemented.
 */
export async function withSpan<T>(
  name: string,
  fn: (span: SpanHandle) => Promise<T> | T,
  attributes?: Record<string, string | number | boolean>,
): Promise<T> {
  throw new Error(
    `spans.ts: withSpan() not implemented (name=${name}, attributes=${JSON.stringify(attributes)}, fn=${typeof fn})`,
  );
}
