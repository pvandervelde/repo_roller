/**
 * RED tests for the SvelteKit server-hop trace continuation gap (Task #3,
 * frontend half, follow-up audit finding):
 *
 * `src/hooks.server.ts`'s `handle` hook never extracts the inbound
 * `traceparent` header the browser sends, and never creates an active OTel
 * span as a child of it. Consequence: `src/lib/proxy.server.ts`'s
 * `proxyToBackend()` → `getCurrentTraceparent()` call (made later in the
 * SAME request, while handling a proxied `/api/v1/**` route) almost never
 * finds an active span to continue, and mints a brand-new, DISCONNECTED
 * root trace instead of continuing the browser's trace. The browser's trace
 * and the backend's trace end up as two separate traces in Jaeger, not one
 * continuous chain — defeating the whole point of Task #3.
 *
 * These tests exercise `handle` directly (not through a real SvelteKit
 * server), building a minimal fake `RequestEvent` that only touches the
 * properties `handle` itself reads/writes (`cookies`, `locals`, `request`,
 * `url`) — consistent with this file testing the hook's CONTRACT, not
 * SvelteKit's routing machinery.
 *
 * Deliberately UNMOCKED: `hooks.server.ts`, `observability.server.ts`, and
 * `trace-context.ts` all run for real here (no `vi.mock`). This is load-
 * bearing, not incidental — `hooks.server.ts` calls `initServerObservability()`
 * at module load, and `NodeTracerProvider#register()` (called there with no
 * arguments) installs BOTH a real W3C `traceparent`-decoding propagator AND
 * an `AsyncLocalStorageContextManager` globally by default (confirmed by
 * reading `@opentelemetry/sdk-trace-node`'s `NodeTracerProvider.register()`
 * source: `config.propagator`/`config.contextManager` being `undefined`
 * triggers its own "install the default" branch). Mocking any of these three
 * modules would hide the exact wiring gap this suite exists to prove.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import fc from 'fast-check';
import { trace } from '@opentelemetry/api';
import type { RequestEvent } from '@sveltejs/kit';
import { handle } from '../src/hooks.server';
import { getCurrentTraceparent, TRACEPARENT_FORMAT } from '../src/lib/observability/trace-context';
import { signSessionCookie } from '../src/lib/session.server';
import type { Session } from '../src/lib/types/session';

// ---------------------------------------------------------------------------
// Fixtures / helpers
// ---------------------------------------------------------------------------

const INBOUND_TRACE_ID = '1'.repeat(32);
const INBOUND_SPAN_ID = '2'.repeat(16);
const INBOUND_HEADER = `00-${INBOUND_TRACE_ID}-${INBOUND_SPAN_ID}-01`;

const SESSION_SECRET = 'a'.repeat(32);

type HandleArgs = Parameters<typeof handle>[0];

interface FakeCookies {
  get: (name: string) => string | undefined;
  delete: (name: string, opts?: { path: string }) => void;
  deleteCalls: Array<{ name: string; opts?: { path: string } }>;
}

function buildFakeCookies(sessionCookieValue?: string): FakeCookies {
  const deleteCalls: FakeCookies['deleteCalls'] = [];
  return {
    get: (name: string) => (name === 'session' ? sessionCookieValue : undefined),
    delete: (name: string, opts?: { path: string }) => {
      deleteCalls.push({ name, opts });
    },
    deleteCalls,
  };
}

/**
 * Build a minimal fake `RequestEvent` exposing only the properties
 * `hooks.server.ts`'s `handle` actually reads or writes: `cookies`,
 * `locals`, `request`, `url`. Cast through `unknown` since the real
 * `RequestEvent` type carries many more fields this hook never touches.
 */
function buildFakeEvent(request: Request, cookies: FakeCookies): RequestEvent {
  const locals: Partial<App.Locals> = {};
  return {
    cookies,
    locals,
    request,
    url: new URL(request.url),
  } as unknown as RequestEvent;
}

function buildValidSessionCookie(session: Session): string {
  return signSessionCookie(session as unknown as Record<string, unknown>);
}

beforeEach(() => {
  vi.stubEnv('SESSION_SECRET', SESSION_SECRET);
});

afterEach(() => {
  vi.unstubAllEnvs();
});

// ---------------------------------------------------------------------------
// Tier 1 — specification: inbound traceparent is continued into resolve()
// ---------------------------------------------------------------------------

describe('handle() — continues an inbound traceparent into resolve() (specification)', () => {
  it('a downstream getCurrentTraceparent() call during resolve() carries the SAME trace id as the inbound header', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies());
    let captured = '';
    const resolve = async (): Promise<Response> => {
      captured = getCurrentTraceparent();
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(captured.split('-')[1]).toBe(INBOUND_TRACE_ID);
  });

  it('a downstream getCurrentTraceparent() call during resolve() carries a DIFFERENT span id than the inbound header (new child span for the SvelteKit hop)', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies());
    let captured = '';
    const resolve = async (): Promise<Response> => {
      captured = getCurrentTraceparent();
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(captured.split('-')[2]).not.toBe(INBOUND_SPAN_ID);
  });

  it('the captured downstream traceparent is itself well-formed', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies());
    let captured = '';
    const resolve = async (): Promise<Response> => {
      captured = getCurrentTraceparent();
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(captured).toMatch(TRACEPARENT_FORMAT);
  });

  it('runs resolve() with an active OTel span in scope (stub-killing: a handle() that just calls resolve(event) with no span wrapping at all fails this)', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies());
    let activeSpanDuringResolve: unknown;
    const resolve = async (): Promise<Response> => {
      activeSpanDuringResolve = trace.getActiveSpan();
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(activeSpanDuringResolve).toBeDefined();
  });
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial: absent header falls back to a fresh root (no regression)
// ---------------------------------------------------------------------------

describe('handle() — absent inbound traceparent falls back to a fresh root span (adversarial)', () => {
  it('still produces a well-formed downstream traceparent when no header is present', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates');
    const event = buildFakeEvent(request, buildFakeCookies());
    let captured = '';
    const resolve = async (): Promise<Response> => {
      captured = getCurrentTraceparent();
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(captured).toMatch(TRACEPARENT_FORMAT);
  });

  it('mints a different trace id on two separate header-less requests (not a hardcoded constant)', async () => {
    const first = { value: '' };
    const second = { value: '' };
    const req1 = new Request('http://localhost/api/v1/orgs/acme/templates');
    const req2 = new Request('http://localhost/api/v1/orgs/acme/other');

    await handle({
      event: buildFakeEvent(req1, buildFakeCookies()),
      resolve: async () => {
        first.value = getCurrentTraceparent();
        return new Response('ok');
      },
    } as HandleArgs);
    await handle({
      event: buildFakeEvent(req2, buildFakeCookies()),
      resolve: async () => {
        second.value = getCurrentTraceparent();
        return new Response('ok');
      },
    } as HandleArgs);

    expect(first.value.split('-')[1]).not.toBe(second.value.split('-')[1]);
  });
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial: malformed inbound traceparent never throws
// ---------------------------------------------------------------------------

describe('handle() — malformed inbound traceparent fails safe (adversarial)', () => {
  const MALFORMED_HEADERS = [
    'not-a-traceparent',
    '00-' + '0'.repeat(32) + '-' + '0'.repeat(16) + '-00', // all-zero, invalid per W3C
    '00-' + 'a'.repeat(31) + '-' + 'b'.repeat(16) + '-01', // trace-id one hex digit short
    'XX-' + 'c'.repeat(32) + '-' + 'd'.repeat(16) + '-01', // non-hex version byte
    '', // present but empty
  ];

  it.each(MALFORMED_HEADERS)(
    'resolves (does not reject) for malformed header %j',
    async (malformed) => {
      const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
        headers: { traceparent: malformed },
      });
      const event = buildFakeEvent(request, buildFakeCookies());
      const resolve = async (): Promise<Response> => new Response('ok');

      await expect(handle({ event, resolve } as HandleArgs)).resolves.toBeInstanceOf(Response);
    },
  );

  it.each(MALFORMED_HEADERS)(
    'still produces a well-formed downstream traceparent for malformed header %j',
    async (malformed) => {
      const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
        headers: { traceparent: malformed },
      });
      const event = buildFakeEvent(request, buildFakeCookies());
      let captured = '';
      const resolve = async (): Promise<Response> => {
        captured = getCurrentTraceparent();
        return new Response('ok');
      };

      await handle({ event, resolve } as HandleArgs);

      expect(captured).toMatch(TRACEPARENT_FORMAT);
    },
  );
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial: wrapping does not alter resolve()'s own behaviour
// ---------------------------------------------------------------------------

describe('handle() — resolve() response and error behaviour is unaffected by span wrapping (adversarial)', () => {
  it('returns the EXACT Response instance produced by resolve(), unmodified', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates');
    const event = buildFakeEvent(request, buildFakeCookies());
    const realResponse = new Response('body', { status: 201, headers: { 'X-Custom': 'yes' } });
    const resolve = async (): Promise<Response> => realResponse;

    const result = await handle({ event, resolve } as HandleArgs);

    expect(result).toBe(realResponse);
  });

  it('propagates the EXACT error thrown by resolve(), unchanged', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates');
    const event = buildFakeEvent(request, buildFakeCookies());
    const boom = new Error('downstream handler exploded');
    const resolve = async (): Promise<Response> => {
      throw boom;
    };

    await expect(handle({ event, resolve } as HandleArgs)).rejects.toBe(boom);
  });

  it('propagates a non-Error rejection value thrown by resolve(), unchanged', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates');
    const event = buildFakeEvent(request, buildFakeCookies());
    const resolve = async (): Promise<Response> => {
      throw 'a string rejection';
    };

    await expect(handle({ event, resolve } as HandleArgs)).rejects.toBe('a string rejection');
  });

  it('calls resolve() exactly once', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates');
    const event = buildFakeEvent(request, buildFakeCookies());
    let callCount = 0;
    const resolve = async (): Promise<Response> => {
      callCount += 1;
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(callCount).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// Tier 2 — adversarial: no active-span leak outside the request's lifetime
// ---------------------------------------------------------------------------

describe('handle() — does not leak an active span beyond the request (adversarial)', () => {
  it('leaves no active span once handle() has resolved successfully', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies());
    const resolve = async (): Promise<Response> => new Response('ok');

    await handle({ event, resolve } as HandleArgs);

    expect(trace.getActiveSpan()).toBeUndefined();
  });

  it('leaves no active span once handle() has rejected', async () => {
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies());
    const resolve = async (): Promise<Response> => {
      throw new Error('boom');
    };

    await Promise.resolve(handle({ event, resolve } as HandleArgs)).catch(() => undefined);

    expect(trace.getActiveSpan()).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Regression — existing auth-guard / session-cookie behaviour is unaffected
// ---------------------------------------------------------------------------

describe('handle() — auth guard regression: protected routes without a session', () => {
  it('redirects to /sign-in without calling resolve()', async () => {
    const request = new Request('http://localhost/create');
    const event = buildFakeEvent(request, buildFakeCookies());
    let resolveCalled = false;
    const resolve = async (): Promise<Response> => {
      resolveCalled = true;
      return new Response('should not get here');
    };

    const thrown = await Promise.resolve(handle({ event, resolve } as HandleArgs)).catch(
      (e: unknown) => e,
    );

    expect(thrown).toMatchObject({ status: 302, location: '/sign-in' });
    expect(resolveCalled).toBe(false);
  });
});

describe('handle() — auth guard regression: protected routes with a valid session', () => {
  it('calls resolve() (no redirect) and populates locals.session', async () => {
    const session: Session = {
      userLogin: 'octocat',
      userAvatarUrl: null,
      backendToken: 'jwt-token-value',
    };
    const cookieValue = buildValidSessionCookie(session);
    const request = new Request('http://localhost/create');
    const event = buildFakeEvent(request, buildFakeCookies(cookieValue));
    let resolveCalled = false;
    const resolve = async (): Promise<Response> => {
      resolveCalled = true;
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(resolveCalled).toBe(true);
    expect(event.locals.session).toEqual(session);
  });
});

describe('handle() — auth guard regression: forged/invalid session cookie is cleared', () => {
  it('clears the cookie, sets locals.session to null, and still calls resolve() on a non-protected route', async () => {
    const request = new Request('http://localhost/');
    const cookies = buildFakeCookies('not-a-valid-signed-cookie');
    const event = buildFakeEvent(request, cookies);
    let resolveCalled = false;
    const resolve = async (): Promise<Response> => {
      resolveCalled = true;
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(event.locals.session).toBeNull();
    expect(cookies.deleteCalls).toHaveLength(1);
    expect(cookies.deleteCalls[0]).toMatchObject({ name: 'session' });
    expect(resolveCalled).toBe(true);
  });
});

describe('handle() — auth guard regression: no cookie at all on a non-protected route', () => {
  it('sets locals.session to null and still calls resolve()', async () => {
    const request = new Request('http://localhost/');
    const event = buildFakeEvent(request, buildFakeCookies());
    let resolveCalled = false;
    const resolve = async (): Promise<Response> => {
      resolveCalled = true;
      return new Response('ok');
    };

    await handle({ event, resolve } as HandleArgs);

    expect(event.locals.session).toBeNull();
    expect(resolveCalled).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Tier 3 — property: for arbitrary inbound header shapes, handle() always
// resolves (never throws for a reason other than the legitimate /sign-in
// redirect tested above) and always leaves behind a well-formed downstream
// traceparent visible to resolve().
// ---------------------------------------------------------------------------

describe('handle() — property: well-formed downstream traceparent for arbitrary inbound header noise', () => {
  const headerArb = fc.oneof(
    fc.constant(''),
    fc.constant(INBOUND_HEADER),
    fc.constant('garbage'),
    fc
      .array(fc.constantFrom(...'0123456789abcdefABCDEF-'.split('')), { maxLength: 60 })
      .map((chars) => chars.join('')),
  );

  it('always leaves a well-formed traceparent visible to resolve(), regardless of inbound header shape', async () => {
    await fc.assert(
      fc.asyncProperty(headerArb, async (headerValue) => {
        const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
          headers: { traceparent: headerValue },
        });
        const event = buildFakeEvent(request, buildFakeCookies());
        let captured = '';
        const resolve = async (): Promise<Response> => {
          captured = getCurrentTraceparent();
          return new Response('ok');
        };

        await handle({ event, resolve } as HandleArgs);

        expect(captured).toMatch(TRACEPARENT_FORMAT);
      }),
      { numRuns: 25 },
    );
  });
});
