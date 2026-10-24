/**
 * RED tests for the follow-up "real OTLP exporter" requirement on
 * `observability.server.ts` (Task #3 frontend observability, completing
 * the end-to-end browser → SvelteKit server → Rust API trace).
 *
 * Two complementary angles, mirroring how the Rust backend's
 * `tracing_otel_tests.rs` separately tests `build_production_otel_layer`
 * (the pure "which exporter for these settings" decision) and
 * `init_tracing_with_otel` (the production entry point that actually wires
 * it up):
 *
 *   1. `buildServerSpanExporter(settings)` tested directly — proves the
 *      exporter-selection decision itself is correct, independent of
 *      `initServerObservability()`'s surrounding SDK registration.
 *   2. `initServerObservability()` tested end-to-end with a spy on
 *      `SimpleSpanProcessor`'s constructor (the point where the exporter
 *      instance is actually handed to the SDK) — proves
 *      `buildServerSpanExporter`'s result is ACTUALLY used by the
 *      production entry point, not just defined-but-orphaned.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ConsoleSpanExporter } from '@opentelemetry/sdk-trace-node';
import { OTLPTraceExporter as GrpcOTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-grpc';
import type { OtelSettings } from '../src/lib/observability/otel-settings';

const { capturedExporters } = vi.hoisted(() => ({
  capturedExporters: [] as unknown[],
}));

vi.mock('@opentelemetry/sdk-trace-node', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@opentelemetry/sdk-trace-node')>();
  class SpyingSimpleSpanProcessor extends actual.SimpleSpanProcessor {
    constructor(exporter: ConstructorParameters<typeof actual.SimpleSpanProcessor>[0]) {
      capturedExporters.push(exporter);
      super(exporter);
    }
  }
  return { ...actual, SimpleSpanProcessor: SpyingSimpleSpanProcessor };
});

const { buildServerSpanExporter, initServerObservability, _resetObservabilityForTesting } =
  await import('../src/lib/observability/observability.server');

const ENABLED_SETTINGS: OtelSettings = {
  enabled: true,
  otlpEndpoint: 'http://jaeger:4317',
  serviceName: 'repo-roller-frontend',
};

const DISABLED_SETTINGS: OtelSettings = {
  enabled: false,
  otlpEndpoint: 'http://jaeger:4317',
  serviceName: 'repo-roller-frontend',
};

beforeEach(() => {
  capturedExporters.length = 0;
  _resetObservabilityForTesting();
});

afterEach(() => {
  vi.unstubAllEnvs();
});

// ---------------------------------------------------------------------------
// buildServerSpanExporter() — direct unit tests
// ---------------------------------------------------------------------------

describe('buildServerSpanExporter() — enabled settings produce a real OTLP exporter', () => {
  it('returns an instance of the gRPC OTLP exporter, not ConsoleSpanExporter', () => {
    const exporter = buildServerSpanExporter(ENABLED_SETTINGS);
    expect(exporter).toBeInstanceOf(GrpcOTLPTraceExporter);
    expect(exporter).not.toBeInstanceOf(ConsoleSpanExporter);
  });

  it('exposes the SpanExporter shape (export/shutdown methods) regardless of exact package choice', () => {
    const exporter = buildServerSpanExporter(ENABLED_SETTINGS) as {
      export?: unknown;
      shutdown?: unknown;
    };
    expect(typeof exporter.export).toBe('function');
    expect(typeof exporter.shutdown).toBe('function');
  });
});

describe('buildServerSpanExporter() — disabled settings are fail-safe-off', () => {
  it('returns a ConsoleSpanExporter when settings.enabled is false', () => {
    const exporter = buildServerSpanExporter(DISABLED_SETTINGS);
    expect(exporter).toBeInstanceOf(ConsoleSpanExporter);
  });

  it('does not return a real OTLP exporter when disabled, even with a configured endpoint', () => {
    const exporter = buildServerSpanExporter(DISABLED_SETTINGS);
    expect(exporter).not.toBeInstanceOf(GrpcOTLPTraceExporter);
  });
});

describe('buildServerSpanExporter() — never throws', () => {
  it('does not throw for enabled settings with a well-formed endpoint', () => {
    expect(() => buildServerSpanExporter(ENABLED_SETTINGS)).not.toThrow();
  });

  it('does not throw for disabled settings', () => {
    expect(() => buildServerSpanExporter(DISABLED_SETTINGS)).not.toThrow();
  });

  it('does not throw for enabled settings with a malformed endpoint (fail-safe-off, not a crash)', () => {
    const malformed: OtelSettings = {
      enabled: true,
      otlpEndpoint: 'not a url at all',
      serviceName: 'svc',
    };
    expect(() => buildServerSpanExporter(malformed)).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// initServerObservability() — end-to-end wiring (stub-killing: catches a
// buildServerSpanExporter that exists but is never actually called).
// ---------------------------------------------------------------------------

describe('initServerObservability() — actually uses buildServerSpanExporter()', () => {
  it('passes a non-ConsoleSpanExporter to SimpleSpanProcessor when OTEL_TRACES_ENABLED=true', () => {
    vi.stubEnv('OTEL_TRACES_ENABLED', 'true');
    vi.stubEnv('OTEL_EXPORTER_OTLP_ENDPOINT', 'http://jaeger:4317');

    initServerObservability();

    expect(capturedExporters).toHaveLength(1);
    expect(capturedExporters[0]).not.toBeInstanceOf(ConsoleSpanExporter);
  });

  it('passes a ConsoleSpanExporter to SimpleSpanProcessor when OTEL_TRACES_ENABLED is unset', () => {
    vi.stubEnv('OTEL_TRACES_ENABLED', undefined);

    initServerObservability();

    expect(capturedExporters).toHaveLength(1);
    expect(capturedExporters[0]).toBeInstanceOf(ConsoleSpanExporter);
  });

  it('passes a ConsoleSpanExporter when OTEL_TRACES_ENABLED is an unrecognised value (fail-safe-off)', () => {
    vi.stubEnv('OTEL_TRACES_ENABLED', 'yes-please');

    initServerObservability();

    expect(capturedExporters).toHaveLength(1);
    expect(capturedExporters[0]).toBeInstanceOf(ConsoleSpanExporter);
  });
});
