/**
 * Mirrors `crates/repo_roller_api/src/tracing_otel_tests.rs` and
 * `tracing_otel_proptests.rs`'s coverage of `resolve_otel_settings`, applied
 * to this module's `resolveOtelSettings` — same env var semantics, same
 * fail-safe-off default, same defaulting rules, translated to TypeScript.
 */
import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import {
  resolveOtelSettings,
  DEFAULT_OTLP_ENDPOINT,
  DEFAULT_SERVICE_NAME,
} from '../src/lib/observability/otel-settings';

// ---------------------------------------------------------------------------
// Tier 1 — specification tests
// ---------------------------------------------------------------------------

describe('resolveOtelSettings() — defaults when everything is unset', () => {
  it('defaults to disabled', () => {
    const settings = resolveOtelSettings(null, null, null);
    expect(settings.enabled).toBe(false);
  });

  it('defaults the endpoint to DEFAULT_OTLP_ENDPOINT', () => {
    const settings = resolveOtelSettings(null, null, null);
    expect(settings.otlpEndpoint).toBe(DEFAULT_OTLP_ENDPOINT);
  });

  it('defaults the service name to DEFAULT_SERVICE_NAME', () => {
    const settings = resolveOtelSettings(null, null, null);
    expect(settings.serviceName).toBe(DEFAULT_SERVICE_NAME);
  });
});

describe('resolveOtelSettings() — enabled flag matching', () => {
  it('enables export for lowercase "true"', () => {
    expect(resolveOtelSettings('true', null, null).enabled).toBe(true);
  });

  it('enables export for mixed-case "True"', () => {
    expect(resolveOtelSettings('True', null, null).enabled).toBe(true);
  });

  it('enables export for "1"', () => {
    expect(resolveOtelSettings('1', null, null).enabled).toBe(true);
  });

  it('stays disabled for "false"', () => {
    expect(resolveOtelSettings('false', null, null).enabled).toBe(false);
  });

  it('stays disabled for "0"', () => {
    expect(resolveOtelSettings('0', null, null).enabled).toBe(false);
  });

  it('stays disabled for an arbitrary garbage value', () => {
    expect(resolveOtelSettings('yes-please', null, null).enabled).toBe(false);
  });

  it('stays disabled for an empty string', () => {
    expect(resolveOtelSettings('', null, null).enabled).toBe(false);
  });

  it('stays disabled for undefined', () => {
    expect(resolveOtelSettings(undefined, null, null).enabled).toBe(false);
  });
});

describe('resolveOtelSettings() — endpoint override', () => {
  it('uses a custom endpoint verbatim', () => {
    const settings = resolveOtelSettings(null, 'http://collector:4317', null);
    expect(settings.otlpEndpoint).toBe('http://collector:4317');
  });

  it('falls back to the default when the endpoint is an empty string', () => {
    const settings = resolveOtelSettings(null, '', null);
    expect(settings.otlpEndpoint).toBe(DEFAULT_OTLP_ENDPOINT);
  });
});

describe('resolveOtelSettings() — service name override', () => {
  it('uses a custom service name verbatim', () => {
    const settings = resolveOtelSettings(null, null, 'my-custom-service');
    expect(settings.serviceName).toBe('my-custom-service');
  });

  it('falls back to the default when the service name is an empty string', () => {
    const settings = resolveOtelSettings(null, null, '');
    expect(settings.serviceName).toBe(DEFAULT_SERVICE_NAME);
  });
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial / boundary tests
// ---------------------------------------------------------------------------

describe('resolveOtelSettings() — adversarial inputs', () => {
  it('never throws on a pathologically long value', () => {
    const raw = 'x'.repeat(10_000);
    expect(() => resolveOtelSettings(raw, raw, raw)).not.toThrow();
  });

  it('never throws on non-ASCII unicode', () => {
    expect(() => resolveOtelSettings('有効', '🚀', 'サービス')).not.toThrow();
  });

  it('treats a whitespace-only enabled value as disabled', () => {
    expect(resolveOtelSettings('   ', null, null).enabled).toBe(false);
  });

  it('two calls with different inputs do not leak state into each other (purity)', () => {
    const disabled = resolveOtelSettings('false', null, null);
    const enabled = resolveOtelSettings('true', null, null);
    expect(disabled.enabled).toBe(false);
    expect(enabled.enabled).toBe(true);
  });

  it('does not mutate its inputs', () => {
    const endpoint = 'http://collector:4317';
    resolveOtelSettings('true', endpoint, 'svc');
    expect(endpoint).toBe('http://collector:4317');
  });
});

// ---------------------------------------------------------------------------
// Tier 3 — property-based tests (fast-check), mirroring
// tracing_otel_proptests.rs's four properties.
// ---------------------------------------------------------------------------

describe('resolveOtelSettings() — properties', () => {
  it('never throws for any combination of arbitrary raw values', () => {
    fc.assert(
      fc.property(
        fc.string({ maxLength: 100 }),
        fc.string({ maxLength: 100 }),
        fc.string({ maxLength: 100 }),
        (enabled, endpoint, serviceName) => {
          expect(() => resolveOtelSettings(enabled, endpoint, serviceName)).not.toThrow();
        },
      ),
      { numRuns: 100 },
    );
  });

  it('only the case-insensitive literals "true"/"1" ever enable export', () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-zA-Z0-9_-]{0,20}$/), (raw) => {
        const settings = resolveOtelSettings(raw, null, null);
        const shouldBeEnabled = raw.toLowerCase() === 'true' || raw === '1';
        expect(settings.enabled).toBe(shouldBeEnabled);
      }),
      { numRuns: 100 },
    );
  });

  it('a non-empty endpoint override is always used verbatim', () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-zA-Z0-9:/.-]{1,50}$/), (endpoint) => {
        const settings = resolveOtelSettings(null, endpoint, null);
        expect(settings.otlpEndpoint).toBe(endpoint);
      }),
      { numRuns: 100 },
    );
  });

  it('a non-empty service name override is always used verbatim', () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[a-zA-Z0-9_-]{1,50}$/), (serviceName) => {
        const settings = resolveOtelSettings(null, null, serviceName);
        expect(settings.serviceName).toBe(serviceName);
      }),
      { numRuns: 100 },
    );
  });
});
