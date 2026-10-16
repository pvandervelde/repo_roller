/**
 * Browser-side OpenTelemetry SDK bootstrap.
 *
 * This module is browser-only. It MUST NOT be imported from server-side
 * code (`*.server.ts`), and it MUST fail fast (throw a clear, descriptive
 * error) if it is ever invoked in a context where `window` is undefined,
 * rather than silently doing nothing — a silent no-op would hide the bug of
 * having been mis-imported into SSR code.
 *
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
 *
 * ### Real OTLP exporter wiring
 * - The browser cannot read `process.env`, so its OTLP endpoint cannot be
 *   resolved internally the way `observability.server.ts` resolves its own
 *   (see `./otel-settings`'s module doc comment for the full explanation of
 *   why the browser needs a SEPARATE, publicly-routable endpoint, delivered
 *   via `$env/dynamic/public`'s `PUBLIC_OTEL_EXPORTER_OTLP_ENDPOINT`). The
 *   caller (`src/hooks.client.ts`) reads that value and passes it through
 *   `initClientObservability({ otlpEndpoint })`.
 * - `initClientObservability(options?)` passes `options?.otlpEndpoint` to
 *   `buildClientSpanExporter()` and uses its result for the SDK's span
 *   processor.
 * - `buildClientSpanExporter(otlpEndpoint)` returns a real OTLP/HTTP
 *   exporter instance (`@opentelemetry/exporter-trace-otlp-http`'s
 *   `OTLPTraceExporter`, constructed with `{ url: otlpEndpoint }`; gRPC is
 *   not usable from a browser at all, so `-grpc` is not an option here) when
 *   `otlpEndpoint` is a non-empty string — fail-safe-off: when
 *   `otlpEndpoint` is `undefined`, `null`, or empty, it returns a
 *   `ConsoleSpanExporter` WITHOUT ever constructing the real OTLP/HTTP
 *   exporter class.
 * - MUST NOT throw. Exporter-construction failure must fall back to
 *   `ConsoleSpanExporter`, not propagate and break the page.
 */

import {
  ConsoleSpanExporter,
  SimpleSpanProcessor,
  WebTracerProvider,
  type SpanExporter,
} from '@opentelemetry/sdk-trace-web';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { FetchInstrumentation } from '@opentelemetry/instrumentation-fetch';

/** Module-level bookkeeping: has init already run (successfully) in this page session? */
let initialized = false;

/**
 * Options controlling browser OTel SDK bootstrap. See this module's doc
 * comment ("Real OTLP exporter wiring") for how `otlpEndpoint` is expected
 * to reach the caller.
 */
export interface ClientObservabilityOptions {
  /**
   * OTLP/HTTP collector endpoint the browser should export spans to (e.g.
   * `http://localhost:4318/v1/traces`). When omitted, `null`, or empty,
   * falls back to `ConsoleSpanExporter` — fail-safe-off, no network export
   * is attempted without an explicitly configured endpoint.
   */
  otlpEndpoint?: string | null;
}

/**
 * Initialise the browser OTel SDK for this page session. Safe to call
 * multiple times; only the first call has an effect. Throws if called
 * outside a browser context (`window` undefined).
 *
 * `options?.otlpEndpoint` is threaded through `buildClientSpanExporter()`
 * to pick a real OTLP/HTTP exporter when configured, or a
 * `ConsoleSpanExporter` fail-safe-off default otherwise.
 */
export function initClientObservability(options?: ClientObservabilityOptions): void {
  // Guard clause MUST run before any OTel registration work — this module is
  // browser-only and must fail loudly (not silently no-op) if mis-imported
  // into SSR code.
  if (typeof window === 'undefined') {
    throw new Error(
      'observability.client.ts: initClientObservability() must only be called in a browser context (window is undefined) — this module is browser-only and must not be imported from server-side code',
    );
  }

  if (initialized) return;

  try {
    const exporter = buildClientSpanExporter(options?.otlpEndpoint) as SpanExporter;
    const provider = new WebTracerProvider({
      spanProcessors: [new SimpleSpanProcessor(exporter)],
    });
    provider.register();

    registerInstrumentations({ instrumentations: [new FetchInstrumentation()] });

    initialized = true;
  } catch {
    // Observability bootstrap must never be able to break the page. Leave
    // `initialized` false so a later call can retry.
  }
}

/**
 * Whether `initClientObservability()` has successfully run in this page
 * session.
 */
export function isClientObservabilityInitialized(): boolean {
  return initialized;
}

/**
 * Test-only hook: reset module-level init state between tests.
 */
export function _resetObservabilityForTesting(): void {
  initialized = false;
}

/**
 * Builds the `SpanExporter` `initClientObservability()` should use, given
 * the resolved browser-facing OTLP endpoint (or its absence).
 *
 * - When `otlpEndpoint` is `undefined`, `null`, or empty, MUST return a
 *   `ConsoleSpanExporter` (or equivalent) WITHOUT constructing any real
 *   network-bound OTLP/HTTP exporter — fail-safe-off.
 * - When `otlpEndpoint` is a non-empty string, MUST return a real OTLP/HTTP
 *   exporter instance pointed at it. See this module's doc comment ("Real
 *   OTLP exporter wiring") for the specific package this contract expects
 *   (`@opentelemetry/exporter-trace-otlp-http`).
 * - MUST NOT throw. Exporter-construction failure must fall back to
 *   `ConsoleSpanExporter`, not propagate.
 */
export function buildClientSpanExporter(otlpEndpoint: string | null | undefined): unknown {
  if (!otlpEndpoint) {
    return new ConsoleSpanExporter();
  }

  try {
    return new OTLPTraceExporter({ url: otlpEndpoint });
  } catch {
    // Exporter construction must never be able to break the page.
    return new ConsoleSpanExporter();
  }
}
