/**
 * RED tests for the follow-up "real OTLP exporter" requirement on
 * `observability.client.ts` (browser side), mirroring
 * `tests/observability-server-otlp.test.ts`'s two-angle approach:
 *
 *   1. `buildClientSpanExporter(otlpEndpoint)` tested directly.
 *   2. `initClientObservability(options)` tested end-to-end with a spy on
 *      `SimpleSpanProcessor`'s constructor, proving the endpoint option is
 *      actually threaded through to the SDK, not just accepted and ignored.
 *
 * The browser cannot read `process.env` — see `otel-settings.ts`'s module
 * doc comment for why the endpoint must instead arrive as an explicit
 * `options.otlpEndpoint` parameter (sourced by the Coder-updated
 * `hooks.client.ts` from `$app/env/public`), rather than being resolved
 * internally by this module the way the server side resolves its own.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ConsoleSpanExporter } from '@opentelemetry/sdk-trace-web';
import { OTLPTraceExporter as HttpOTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';

const { capturedExporters } = vi.hoisted(() => ({
  capturedExporters: [] as unknown[],
}));

vi.mock('@opentelemetry/sdk-trace-web', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@opentelemetry/sdk-trace-web')>();
  class SpyingSimpleSpanProcessor extends actual.SimpleSpanProcessor {
    constructor(exporter: ConstructorParameters<typeof actual.SimpleSpanProcessor>[0]) {
      capturedExporters.push(exporter);
      super(exporter);
    }
  }
  return { ...actual, SimpleSpanProcessor: SpyingSimpleSpanProcessor };
});

const { buildClientSpanExporter, initClientObservability, _resetObservabilityForTesting } =
  await import('../src/lib/observability/observability.client');

const OTLP_HTTP_ENDPOINT = 'http://localhost:4318/v1/traces';

beforeEach(() => {
  capturedExporters.length = 0;
  _resetObservabilityForTesting();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

// ---------------------------------------------------------------------------
// buildClientSpanExporter() — direct unit tests
// ---------------------------------------------------------------------------

describe('buildClientSpanExporter() — a configured endpoint produces a real OTLP/HTTP exporter', () => {
  it('returns an instance of the OTLP/HTTP exporter, not ConsoleSpanExporter', () => {
    const exporter = buildClientSpanExporter(OTLP_HTTP_ENDPOINT);
    expect(exporter).toBeInstanceOf(HttpOTLPTraceExporter);
    expect(exporter).not.toBeInstanceOf(ConsoleSpanExporter);
  });

  it('exposes the SpanExporter shape (export/shutdown methods) regardless of exact package choice', () => {
    const exporter = buildClientSpanExporter(OTLP_HTTP_ENDPOINT) as {
      export?: unknown;
      shutdown?: unknown;
    };
    expect(typeof exporter.export).toBe('function');
    expect(typeof exporter.shutdown).toBe('function');
  });
});

describe('buildClientSpanExporter() — fail-safe-off without a configured endpoint', () => {
  it('returns a ConsoleSpanExporter when the endpoint is undefined', () => {
    expect(buildClientSpanExporter(undefined)).toBeInstanceOf(ConsoleSpanExporter);
  });

  it('returns a ConsoleSpanExporter when the endpoint is null', () => {
    expect(buildClientSpanExporter(null)).toBeInstanceOf(ConsoleSpanExporter);
  });

  it('returns a ConsoleSpanExporter when the endpoint is an empty string', () => {
    expect(buildClientSpanExporter('')).toBeInstanceOf(ConsoleSpanExporter);
  });

  it('never constructs a real OTLP exporter when no endpoint is given', () => {
    expect(buildClientSpanExporter(undefined)).not.toBeInstanceOf(HttpOTLPTraceExporter);
  });
});

describe('buildClientSpanExporter() — never throws', () => {
  it('does not throw for a well-formed endpoint', () => {
    expect(() => buildClientSpanExporter(OTLP_HTTP_ENDPOINT)).not.toThrow();
  });

  it('does not throw when no endpoint is given', () => {
    expect(() => buildClientSpanExporter(undefined)).not.toThrow();
  });

  it('does not throw for a malformed endpoint (fail-safe-off, not a crash)', () => {
    expect(() => buildClientSpanExporter('not a url at all')).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// initClientObservability() — end-to-end wiring (stub-killing: catches a
// buildClientSpanExporter that exists but is never actually called, or an
// `options` parameter that is accepted but silently ignored).
// ---------------------------------------------------------------------------

describe('initClientObservability() — actually uses buildClientSpanExporter()', () => {
  it('passes a non-ConsoleSpanExporter to SimpleSpanProcessor when given an otlpEndpoint option', () => {
    initClientObservability({ otlpEndpoint: OTLP_HTTP_ENDPOINT });

    expect(capturedExporters).toHaveLength(1);
    expect(capturedExporters[0]).not.toBeInstanceOf(ConsoleSpanExporter);
  });

  it('passes a ConsoleSpanExporter to SimpleSpanProcessor when called with no options at all', () => {
    initClientObservability();

    expect(capturedExporters).toHaveLength(1);
    expect(capturedExporters[0]).toBeInstanceOf(ConsoleSpanExporter);
  });

  it('passes a ConsoleSpanExporter when options.otlpEndpoint is explicitly null (fail-safe-off)', () => {
    initClientObservability({ otlpEndpoint: null });

    expect(capturedExporters).toHaveLength(1);
    expect(capturedExporters[0]).toBeInstanceOf(ConsoleSpanExporter);
  });
});
