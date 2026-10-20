import { describe, it, expect, vi } from 'vitest';
import { withSpan, withRequestSpan, SPAN_NAMES } from '../src/lib/observability/spans';

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

// ---------------------------------------------------------------------------
// withRequestSpan() — the SvelteKit-server-hop span helper.
//
// Scoped to the pure behavioural contract only (call-once, value/error
// passthrough, no-throw-on-malformed-input, no header mutation). Real W3C
// trace-id continuation across an inbound `traceparent` header requires a
// globally-registered OTel TracerProvider/ContextManager — this file does
// not trigger that registration (it never imports `hooks.server.ts` or
// `observability.server.ts`), so asserting on real span/trace ids here would
// be order-dependent on whichever other test file's module-load side effect
// happened to run first in the same worker. That end-to-end assertion lives
// in `tests/hooks-server.test.ts` instead, which deliberately DOES trigger
// real SDK registration by importing `hooks.server.ts` for real.
// ---------------------------------------------------------------------------

describe('SPAN_NAMES.SVELTEKIT_REQUEST — specification', () => {
  it('defines a non-empty name', () => {
    expect(SPAN_NAMES.SVELTEKIT_REQUEST.length).toBeGreaterThan(0);
  });

  it('is distinct from the other span names', () => {
    expect(SPAN_NAMES.SVELTEKIT_REQUEST).not.toBe(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT);
    expect(SPAN_NAMES.SVELTEKIT_REQUEST).not.toBe(SPAN_NAMES.LOAD_TEMPLATES);
  });
});

describe('withRequestSpan() — specification', () => {
  it('resolves with the exact value returned by fn', async () => {
    const request = new Request('http://localhost/');
    const result = await withRequestSpan(request, () => 'the-value');
    expect(result).toBe('the-value');
  });

  it('resolves with the exact value awaited from an async fn', async () => {
    const request = new Request('http://localhost/');
    const result = await withRequestSpan(request, async () => {
      await Promise.resolve();
      return 42;
    });
    expect(result).toBe(42);
  });

  it('calls fn exactly once', async () => {
    const request = new Request('http://localhost/');
    const fn = vi.fn().mockReturnValue('ok');
    await withRequestSpan(request, fn);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('passes a span handle object to fn', async () => {
    const request = new Request('http://localhost/');
    let receivedHandle: unknown;
    await withRequestSpan(request, (span) => {
      receivedHandle = span;
      return null;
    });
    expect(receivedHandle).toBeDefined();
  });
});

describe('withRequestSpan() — resolves regardless of inbound traceparent header shape (adversarial)', () => {
  it('resolves for a request with no traceparent header at all', async () => {
    const request = new Request('http://localhost/');
    await expect(withRequestSpan(request, () => 'ok')).resolves.toBe('ok');
  });

  it('resolves for a request with a well-formed traceparent header', async () => {
    const request = new Request('http://localhost/', {
      headers: { traceparent: '00-' + '1'.repeat(32) + '-' + '2'.repeat(16) + '-01' },
    });
    await expect(withRequestSpan(request, () => 'ok')).resolves.toBe('ok');
  });

  it('resolves for a request with a malformed traceparent header, rather than throwing', async () => {
    const request = new Request('http://localhost/', {
      headers: { traceparent: 'not-a-traceparent' },
    });
    await expect(withRequestSpan(request, () => 'ok')).resolves.toBe('ok');
  });

  it('resolves for a request with an all-zero (spec-invalid) traceparent header', async () => {
    const request = new Request('http://localhost/', {
      headers: { traceparent: '00-' + '0'.repeat(32) + '-' + '0'.repeat(16) + '-00' },
    });
    await expect(withRequestSpan(request, () => 'ok')).resolves.toBe('ok');
  });
});

describe('withRequestSpan() — error propagation (adversarial)', () => {
  it('rethrows the exact error instance thrown synchronously by fn', async () => {
    const request = new Request('http://localhost/');
    const boom = new Error('synchronous boom');
    await expect(
      withRequestSpan(request, () => {
        throw boom;
      }),
    ).rejects.toBe(boom);
  });

  it('rethrows the exact error instance from a rejected async fn', async () => {
    const request = new Request('http://localhost/');
    const boom = new Error('async boom');
    await expect(
      withRequestSpan(request, async () => {
        throw boom;
      }),
    ).rejects.toBe(boom);
  });

  it('does not swallow a non-Error rejection value', async () => {
    const request = new Request('http://localhost/');
    await expect(
      withRequestSpan(request, async () => {
        throw 'string rejection';
      }),
    ).rejects.toBe('string rejection');
  });

  it('calls fn exactly once even when it throws', async () => {
    const request = new Request('http://localhost/');
    const fn = vi.fn().mockImplementation(() => {
      throw new Error('boom');
    });
    await expect(withRequestSpan(request, fn)).rejects.toThrow();
    expect(fn).toHaveBeenCalledTimes(1);
  });
});

describe('withRequestSpan() — does not mutate the given Request (adversarial)', () => {
  it('leaves the original traceparent header value on the request unchanged', async () => {
    const original = '00-' + '3'.repeat(32) + '-' + '4'.repeat(16) + '-01';
    const request = new Request('http://localhost/', {
      headers: { traceparent: original },
    });

    await withRequestSpan(request, () => 'ok');

    expect(request.headers.get('traceparent')).toBe(original);
  });

  it('does not add a traceparent header to a request that had none', async () => {
    const request = new Request('http://localhost/');

    await withRequestSpan(request, () => 'ok');

    expect(request.headers.get('traceparent')).toBeNull();
  });
});
