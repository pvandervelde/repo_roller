import { describe, it, expect, beforeEach } from 'vitest';
import {
  initServerObservability,
  isServerObservabilityInitialized,
  _resetObservabilityForTesting,
} from '../src/lib/observability/observability.server';

beforeEach(() => {
  _resetObservabilityForTesting();
});

describe('isServerObservabilityInitialized() — specification', () => {
  it('reports false before initServerObservability() has been called', () => {
    expect(isServerObservabilityInitialized()).toBe(false);
  });

  it('reports true after initServerObservability() has been called', () => {
    initServerObservability();
    expect(isServerObservabilityInitialized()).toBe(true);
  });
});

describe('initServerObservability() — idempotency (adversarial)', () => {
  it('does not throw when called a second time in the same process', () => {
    initServerObservability();
    expect(() => initServerObservability()).not.toThrow();
  });

  it('does not throw when called three times in a row', () => {
    initServerObservability();
    initServerObservability();
    expect(() => initServerObservability()).not.toThrow();
  });

  it('leaves isServerObservabilityInitialized() true after a redundant second call', () => {
    initServerObservability();
    initServerObservability();
    expect(isServerObservabilityInitialized()).toBe(true);
  });
});

describe('initServerObservability() — resilience to missing config (adversarial)', () => {
  it('does not throw when OTEL_EXPORTER_OTLP_ENDPOINT is unset', () => {
    delete process.env.OTEL_EXPORTER_OTLP_ENDPOINT;
    expect(() => initServerObservability()).not.toThrow();
  });
});

describe('_resetObservabilityForTesting() — state isolation', () => {
  it('causes isServerObservabilityInitialized() to report false again after a prior init', () => {
    initServerObservability();
    expect(isServerObservabilityInitialized()).toBe(true);
    _resetObservabilityForTesting();
    expect(isServerObservabilityInitialized()).toBe(false);
  });
});
