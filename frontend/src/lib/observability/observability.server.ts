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

import {
  ConsoleSpanExporter,
  NodeTracerProvider,
  SimpleSpanProcessor,
} from '@opentelemetry/sdk-trace-node';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { UndiciInstrumentation } from '@opentelemetry/instrumentation-undici';
import { logger } from '../logger.server';

/** Module-level bookkeeping: has init already run (successfully) in this process? */
let initialized = false;

/**
 * Initialise the Node OTel SDK for this process. Safe to call multiple
 * times; only the first call has an effect.
 *
 * No OTLP exporter is wired up yet (see `OTEL_EXPORTER_OTLP_ENDPOINT` below)
 * — spans are currently exported via `ConsoleSpanExporter` as a safe,
 * dependency-free default. Swapping in a real OTLP exporter is tracked as
 * follow-up work once an exporter package is selected.
 */
export function initServerObservability(): void {
  if (initialized) return;

  try {
    const otlpEndpoint = process.env.OTEL_EXPORTER_OTLP_ENDPOINT;
    const provider = new NodeTracerProvider({
      spanProcessors: [new SimpleSpanProcessor(new ConsoleSpanExporter())],
    });
    provider.register();

    registerInstrumentations({ instrumentations: [new UndiciInstrumentation()] });

    initialized = true;
    logger.info('Server observability initialized', {
      otlpEndpointConfigured: Boolean(otlpEndpoint),
    });
  } catch (err) {
    // Observability bootstrap must never be able to crash server startup.
    // Leave `initialized` false so a later call can retry.
    logger.warn('Server observability initialization failed; continuing without tracing', {
      err,
    });
  }
}

/**
 * Whether `initServerObservability()` has successfully run in this process.
 */
export function isServerObservabilityInitialized(): boolean {
  return initialized;
}

/**
 * Test-only hook: reset module-level init state between tests so each test
 * can observe `initServerObservability()`'s first-call behaviour freshly.
 * Has no effect on any already-registered global OTel provider beyond this
 * module's own bookkeeping flag.
 */
export function _resetObservabilityForTesting(): void {
  initialized = false;
}
