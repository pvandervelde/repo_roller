import { describe, it, expect, vi } from 'vitest';
import { withSpan, SPAN_NAMES } from '../src/lib/observability/spans';

describe('SPAN_NAMES — specification', () => {
  it('defines a non-empty name for CREATE_REPOSITORY_SUBMIT', () => {
    expect(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT.length).toBeGreaterThan(0);
  });

  it('defines a non-empty name for LOAD_TEMPLATES', () => {
    expect(SPAN_NAMES.LOAD_TEMPLATES.length).toBeGreaterThan(0);
  });

  it('uses distinct names for distinct operations', () => {
    expect(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT).not.toBe(SPAN_NAMES.LOAD_TEMPLATES);
  });
});

describe('withSpan() — specification', () => {
  it('resolves with the exact value returned by fn', async () => {
    const result = await withSpan(SPAN_NAMES.LOAD_TEMPLATES, () => 'the-value');
    expect(result).toBe('the-value');
  });

  it('resolves with the exact value awaited from an async fn', async () => {
    const result = await withSpan(SPAN_NAMES.LOAD_TEMPLATES, async () => {
      await Promise.resolve();
      return 42;
    });
    expect(result).toBe(42);
  });

  it('calls fn exactly once', async () => {
    const fn = vi.fn().mockReturnValue('ok');
    await withSpan(SPAN_NAMES.LOAD_TEMPLATES, fn);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('passes a span handle object to fn', async () => {
    let receivedHandle: unknown;
    await withSpan(SPAN_NAMES.LOAD_TEMPLATES, (span) => {
      receivedHandle = span;
      return null;
    });
    expect(receivedHandle).toBeDefined();
  });
});

describe('withSpan() — error propagation (adversarial)', () => {
  it('rethrows the exact error instance thrown synchronously by fn', async () => {
    const boom = new Error('synchronous boom');
    await expect(
      withSpan(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT, () => {
        throw boom;
      }),
    ).rejects.toBe(boom);
  });

  it('rethrows the exact error instance from a rejected async fn', async () => {
    const boom = new Error('async boom');
    await expect(
      withSpan(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT, async () => {
        throw boom;
      }),
    ).rejects.toBe(boom);
  });

  it('does not swallow a non-Error rejection value', async () => {
    await expect(
      withSpan(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT, async () => {
        throw 'string rejection';
      }),
    ).rejects.toBe('string rejection');
  });

  it('calls fn exactly once even when it throws', async () => {
    const fn = vi.fn().mockImplementation(() => {
      throw new Error('boom');
    });
    await expect(withSpan(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT, fn)).rejects.toThrow();
    expect(fn).toHaveBeenCalledTimes(1);
  });
});

describe('withSpan() — attributes parameter (boundary)', () => {
  it('accepts an empty attributes object without throwing for that reason', async () => {
    await expect(withSpan(SPAN_NAMES.LOAD_TEMPLATES, () => 'ok', {})).resolves.toBe('ok');
  });

  it('accepts a populated attributes object without throwing for that reason', async () => {
    await expect(
      withSpan(SPAN_NAMES.LOAD_TEMPLATES, () => 'ok', { organization: 'acme', count: 3 }),
    ).resolves.toBe('ok');
  });

  it('works when attributes is omitted entirely', async () => {
    await expect(withSpan(SPAN_NAMES.LOAD_TEMPLATES, () => 'ok')).resolves.toBe('ok');
  });
});
