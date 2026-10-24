/**
 * Node-side OpenTelemetry SDK bootstrap for the SvelteKit server runtime.
 *
 * MUST NOT be imported from client-side code (server-only).
 *
 * - `initServerObservability()` registers the Node OTel SDK (a trace
 *   provider, plus instrumentation for outbound HTTP/undici requests so that
 *   `proxy.server.ts`'s outbound fetch to the Rust backend is automatically
 *   captured as a child span) exactly once per process. It resolves
 *   {@link OtelSettings} via `resolveOtelSettings()` (from `./otel-settings`)
 *   reading `OTEL_TRACES_ENABLED`/`OTEL_EXPORTER_OTLP_ENDPOINT`/
 *   `OTEL_SERVICE_NAME` directly from `process.env`, then passes the result
 *   to `buildServerSpanExporter(settings)` to obtain the exporter handed to
 *   `SimpleSpanProcessor`/`NodeTracerProvider`.
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
 * ### Real OTLP exporter wiring
 * - `buildServerSpanExporter(settings)` returns a real OTLP exporter
 *   instance (`@opentelemetry/exporter-trace-otlp-grpc`'s
 *   `OTLPTraceExporter`, constructed with `{ url: settings.otlpEndpoint }` —
 *   chosen for transport symmetry with the Rust backend's own gRPC/4317 OTLP
 *   exporter and with `examples/local-stack/docker-compose.yml`'s Jaeger
 *   gRPC receiver port) **only when `settings.enabled` is `true`** —
 *   fail-safe-off: when `false`, it returns a `ConsoleSpanExporter` WITHOUT
 *   ever constructing/touching the real OTLP exporter class, so that no
 *   network connection is attempted when nobody asked for one.
 * - Exporter construction failure (e.g. a malformed endpoint, if the chosen
 *   exporter package validates eagerly) is treated as fail-safe-off (fall
 *   back to `ConsoleSpanExporter`) rather than thrown — mirrors the Rust
 *   precedent's `build_production_otel_layer` catch-and-log-not-panic
 *   behaviour.
 */

import {
  ConsoleSpanExporter,
  NodeTracerProvider,
  SimpleSpanProcessor,
  type SpanExporter,
} from '@opentelemetry/sdk-trace-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import { registerInstrumentations } from '@opentelemetry/instrumentation';
import { UndiciInstrumentation } from '@opentelemetry/instrumentation-undici';
import { logger } from '../logger.server';
import { buildFailSafeSpanExporter } from './exporter-fallback';
import { resolveOtelSettings, type OtelSettings } from './otel-settings';

/** Module-level bookkeeping: has init already run (successfully) in this process? */
let initialized = false;

/**
 * Initialise the Node OTel SDK for this process. Safe to call multiple
 * times; only the first call has an effect.
 *
 * Resolves {@link OtelSettings} from `process.env` and uses
 * `buildServerSpanExporter()` to pick a real OTLP exporter when enabled, or
 * a `ConsoleSpanExporter` fail-safe-off default otherwise.
 */
export function initServerObservability(): void {
  if (initialized) return;

  try {
    const settings = resolveOtelSettings(
      process.env.OTEL_TRACES_ENABLED,
      process.env.OTEL_EXPORTER_OTLP_ENDPOINT,
      process.env.OTEL_SERVICE_NAME,
    );
    const exporter = buildServerSpanExporter(settings) as SpanExporter;
    const provider = new NodeTracerProvider({
      spanProcessors: [new SimpleSpanProcessor(exporter)],
    });
    provider.register();

    registerInstrumentations({ instrumentations: [new UndiciInstrumentation()] });

    initialized = true;
    logger.info('Server observability initialized', {
      otlpEndpointConfigured: settings.enabled,
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
 */
export function buildServerSpanExporter(settings: OtelSettings): unknown {
  return buildFailSafeSpanExporter<SpanExporter>(
    settings.enabled,
    () => new OTLPTraceExporter({ url: settings.otlpEndpoint }),
    () => new ConsoleSpanExporter(),
    (err) =>
      logger.warn('Failed to build OTLP span exporter; falling back to ConsoleSpanExporter', {
        err,
      }),
  );
}
