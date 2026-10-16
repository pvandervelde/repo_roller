/**
 * Node-side OpenTelemetry SDK bootstrap for the SvelteKit server runtime.
 *
 * STATUS: PARTIALLY IMPLEMENTED. `initServerObservability` and
 * `isServerObservabilityInitialized` are real, working code (idempotency
 * and browser/SSR guards are correct and covered by
 * `tests/observability-server.test.ts`). `buildServerSpanExporter` below is
 * a NEW stub — it currently throws — defining the contract for the
 * still-outstanding "real OTLP exporter" requirement; see
 * `tests/observability-server-otlp.test.ts`.
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
 *
 * ### Real OTLP exporter wiring (follow-up requirement, RED as of this
 * ### comment — see `tests/otel-settings.test.ts` and
 * ### `tests/observability-server-otlp.test.ts`)
 * - `initServerObservability()` MUST resolve {@link OtelSettings} via
 *   `resolveOtelSettings(process.env.OTEL_TRACES_ENABLED,
 *   process.env.OTEL_EXPORTER_OTLP_ENDPOINT, process.env.OTEL_SERVICE_NAME)`
 *   (from `./otel-settings`) and pass the result to
 *   `buildServerSpanExporter(settings)` to obtain the exporter it hands to
 *   `SimpleSpanProcessor`/`NodeTracerProvider` — replacing today's
 *   unconditional `new ConsoleSpanExporter()`.
 * - `buildServerSpanExporter(settings)` MUST return a real OTLP exporter
 *   instance (this contract's reference implementation uses
 *   `@opentelemetry/exporter-trace-otlp-grpc`'s `OTLPTraceExporter`,
 *   constructed with `{ url: settings.otlpEndpoint }` — chosen for
 *   transport symmetry with the Rust backend's own gRPC/4317 OTLP exporter
 *   and with `examples/local-stack/docker-compose.yml`'s Jaeger gRPC
 *   receiver port; `-http`/`-proto` are acceptable alternatives if gRPC
 *   proves awkward in this Node runtime, but the test suite's `instanceof`
 *   assertions would need updating to match) **only when
 *   `settings.enabled` is `true`** — fail-safe-off: when `false`, it MUST
 *   return a `ConsoleSpanExporter` (or equivalent no-network-I/O stand-in)
 *   WITHOUT ever constructing/touching the real OTLP exporter class, so
 *   that no network connection is attempted when nobody asked for one.
 * - Exporter construction failure (e.g. a malformed endpoint, if the chosen
 *   exporter package validates eagerly) MUST be treated as fail-safe-off
 *   (fall back to `ConsoleSpanExporter`) rather than thrown — mirrors the
 *   Rust precedent's `build_production_otel_layer` catch-and-log-not-panic
 *   behaviour.
 */

import {
  ConsoleSpanExporter,
  NodeTracerProvider,
  SimpleSpanProcessor,
} from '@opentelemetry/sdk-trace-node';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { UndiciInstrumentation } from '@opentelemetry/instrumentation-undici';
import { logger } from '../logger.server';
import type { OtelSettings } from './otel-settings';

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

/**
 * Builds the `SpanExporter` `initServerObservability()` should use, given
 * resolved {@link OtelSettings}.
 *
 * - When `settings.enabled` is `false`, MUST return a `ConsoleSpanExporter`
 *   (or equivalent) WITHOUT constructing any real network-bound OTLP
 *   exporter — fail-safe-off, no network I/O attempted when export wasn't
 *   explicitly turned on.
 * - When `settings.enabled` is `true`, MUST return a real OTLP exporter
 *   instance configured to send spans to `settings.otlpEndpoint`. See this
 *   module's doc comment ("Real OTLP exporter wiring") for the specific
 *   package this contract expects (`@opentelemetry/exporter-trace-otlp-grpc`).
 * - MUST NOT throw. Exporter-construction failure must fall back to
 *   `ConsoleSpanExporter` (fail-safe-off), not propagate.
 *
 * STUB: throws until implemented.
 */
export function buildServerSpanExporter(settings: OtelSettings): unknown {
  throw new Error(
    `observability.server.ts: buildServerSpanExporter() not implemented (settings=${JSON.stringify(settings)})`,
  );
}
