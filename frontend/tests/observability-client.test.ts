import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  initClientObservability,
  isClientObservabilityInitialized,
  _resetObservabilityForTesting,
} from '../src/lib/observability/observability.client';

beforeEach(() => {
  _resetObservabilityForTesting();
});

describe('isClientObservabilityInitialized() — specification', () => {
  it('reports false before initClientObservability() has been called', () => {
    expect(isClientObservabilityInitialized()).toBe(false);
  });

  it('reports true after initClientObservability() has been called', () => {
    initClientObservability();
    expect(isClientObservabilityInitialized()).toBe(true);
  });
});

describe('initClientObservability() — idempotency (adversarial)', () => {
  it('does not throw when called a second time in the same page session', () => {
    initClientObservability();
    expect(() => initClientObservability()).not.toThrow();
  });

  it('does not throw when called three times in a row', () => {
    initClientObservability();
    initClientObservability();
    expect(() => initClientObservability()).not.toThrow();
  });

  it('leaves isClientObservabilityInitialized() true after a redundant second call', () => {
    initClientObservability();
    initClientObservability();
    expect(isClientObservabilityInitialized()).toBe(true);
  });
});

describe('initClientObservability() — browser-only guard (adversarial)', () => {
  const originalWindow = globalThis.window;

  afterEach(() => {
    // Restore jsdom's window for subsequent tests in this file/suite.
    globalThis.window = originalWindow;
  });

  it('throws an error whose message references "browser" when window is undefined', () => {
    // @ts-expect-error -- deliberately simulating an SSR context for this test
    delete globalThis.window;
    expect(() => initClientObservability()).toThrow(/browser/i);
  });

  it('does not throw for the generic "not implemented" stub reason when window is undefined', () => {
    // This pins down *which* error is expected: a real implementation must
    // fail with a browser-context guard error, not merely leave the
    // pre-existing "not implemented" stub message unchanged.
    // @ts-expect-error -- deliberately simulating an SSR context for this test
    delete globalThis.window;
    expect(() => initClientObservability()).not.toThrow(/not implemented/i);
  });
});

describe('_resetObservabilityForTesting() — state isolation', () => {
  it('causes isClientObservabilityInitialized() to report false again after a prior init', () => {
    initClientObservability();
    expect(isClientObservabilityInitialized()).toBe(true);
    _resetObservabilityForTesting();
    expect(isClientObservabilityInitialized()).toBe(false);
  });
});
