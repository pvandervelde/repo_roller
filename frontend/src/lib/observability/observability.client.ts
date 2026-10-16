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

import {
  ConsoleSpanExporter,
  SimpleSpanProcessor,
  WebTracerProvider,
} from '@opentelemetry/sdk-trace-web';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { FetchInstrumentation } from '@opentelemetry/instrumentation-fetch';

/** Module-level bookkeeping: has init already run (successfully) in this page session? */
let initialized = false;

/**
 * Initialise the browser OTel SDK for this page session. Safe to call
 * multiple times; only the first call has an effect. Throws if called
 * outside a browser context (`window` undefined).
 *
 * No OTLP exporter is wired up yet — spans are currently exported via
 * `ConsoleSpanExporter` as a safe, dependency-free default. Swapping in a
 * real OTLP exporter is tracked as follow-up work once an exporter package
 * is selected.
 */
export function initClientObservability(): void {
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
    const provider = new WebTracerProvider({
      spanProcessors: [new SimpleSpanProcessor(new ConsoleSpanExporter())],
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
