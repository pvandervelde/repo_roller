/**
 * End-to-end composition test for the browser -> SvelteKit-server ->
 * Rust-backend trace continuation chain (QA audit, post-REFACTOR gate).
 *
 * `tests/hooks-server.test.ts` proves `handle()` makes a well-formed,
 * trace-id-continuing traceparent visible to `getCurrentTraceparent()` during
 * `resolve()`. `tests/proxy-trace-propagation.test.ts` proves `proxyToBackend()`
 * calls `getCurrentTraceparent()` and attaches its result to the outbound
 * fetch. Neither file ever COMPOSES the two: `hooks-server.test.ts`'s
 * `resolve()` stubs call `getCurrentTraceparent()` directly rather than going
 * through the real `proxyToBackend()`, and `proxy-trace-propagation.test.ts`
 * calls `proxyToBackend()` directly, never from inside a `handle()`-managed
 * request (so it never runs under the `withRequestSpan` active-span context
 * `handle()` establishes).
 *
 * This file closes that gap: it drives a request through the REAL `handle()`
 * hook (real `withRequestSpan`, real OTel SDK registration, real
 * `trace-context.ts`), with a `resolve()` that calls the REAL
 * `proxyToBackend()` (real `getCurrentTraceparent()`), and asserts on the
 * header actually attached to the mocked outbound `fetch()` call — the exact
 * shape a real browser request takes in production. This is the single most
 * important behaviour the whole fix exists to deliver: one continuous trace
 * id from browser to backend, with a distinct span id per hop.
 *
 * Deliberately UNMOCKED: `hooks.server.ts`, `observability.server.ts`,
 * `spans.ts`, and `trace-context.ts`. Only `$env/dynamic/private` (a
 * SvelteKit virtual module baked in at module-resolution time, see the
 * comment in `proxy-trace-propagation.test.ts`) and the global `fetch` are
 * mocked — the minimum needed to run `proxyToBackend()` against a fake
 * backend instead of a real network call.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { RequestEvent } from '@sveltejs/kit';
import { TRACEPARENT_FORMAT } from '../src/lib/observability/trace-context';
import { signSessionCookie } from '../src/lib/session.server';
import type { Session } from '../src/lib/types/session';

const { mockEnv } = vi.hoisted(() => ({
  mockEnv: { BACKEND_API_URL: 'http://backend.internal:3000' } as Record<
    string,
    string | undefined
  >,
}));

vi.mock('$env/dynamic/private', () => ({ env: mockEnv }));

const { handle } = await import('../src/hooks.server');
const { proxyToBackend } = await import('../src/lib/proxy.server');

const INBOUND_TRACE_ID = '9'.repeat(32);
const INBOUND_SPAN_ID = '8'.repeat(16);
const INBOUND_HEADER = `00-${INBOUND_TRACE_ID}-${INBOUND_SPAN_ID}-01`;

const SESSION_SECRET = 'a'.repeat(32);

const SESSION: Session = {
  userLogin: 'octocat',
  userAvatarUrl: null,
  backendToken: 'backend-jwt-value',
};

type HandleArgs = Parameters<typeof handle>[0];

interface FakeCookies {
  get: (name: string) => string | undefined;
  delete: (name: string, opts?: { path: string }) => void;
}

function buildFakeCookies(sessionCookieValue?: string): FakeCookies {
  return {
    get: (name: string) => (name === 'session' ? sessionCookieValue : undefined),
    delete: () => undefined,
  };
}

function buildFakeEvent(request: Request, cookies: FakeCookies): RequestEvent {
  const locals: Partial<App.Locals> = {};
  return {
    cookies,
    locals,
    request,
    url: new URL(request.url),
  } as unknown as RequestEvent;
}

function mockBackendFetchOk(): void {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockImplementation(async () => new Response('{}', { status: 200 })),
  );
}

beforeEach(() => {
  vi.stubEnv('SESSION_SECRET', SESSION_SECRET);
  vi.unstubAllGlobals();
  mockEnv.BACKEND_API_URL = 'http://backend.internal:3000';
});

afterEach(() => {
  vi.unstubAllEnvs();
});

describe('end-to-end: browser traceparent survives handle() -> resolve() -> proxyToBackend() -> outbound fetch', () => {
  it('attaches an outbound traceparent with the SAME trace id and a DIFFERENT span id than the inbound browser header', async () => {
    mockBackendFetchOk();
    const cookieValue = signSessionCookie(SESSION as unknown as Record<string, unknown>);
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies(cookieValue));
    const resolve = async (): Promise<Response> =>
      proxyToBackend(
        '/api/v1/orgs/acme/templates',
        event.request,
        event.locals.session as Session,
      );

    const response = await handle({ event, resolve } as HandleArgs);

    expect(response.status).toBe(200);
    const fetchMock = vi.mocked(fetch);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const call = fetchMock.mock.calls[0];
    const outboundHeaders = new Headers(call[1]?.headers);
    const outboundTraceparent = outboundHeaders.get('traceparent') ?? '';

    expect(outboundTraceparent).toMatch(TRACEPARENT_FORMAT);
    // Same trace id: the browser's original trace survives both hops.
    expect(outboundTraceparent.split('-')[1]).toBe(INBOUND_TRACE_ID);
    // Different span id: the SvelteKit hop is a genuine new child span, not
    // a passthrough of the browser's own span id.
    expect(outboundTraceparent.split('-')[2]).not.toBe(INBOUND_SPAN_ID);
  });

  it('still produces a well-formed outbound traceparent (fresh root) when the browser sends no traceparent at all', async () => {
    mockBackendFetchOk();
    const cookieValue = signSessionCookie(SESSION as unknown as Record<string, unknown>);
    const request = new Request('http://localhost/api/v1/orgs/acme/templates');
    const event = buildFakeEvent(request, buildFakeCookies(cookieValue));
    const resolve = async (): Promise<Response> =>
      proxyToBackend(
        '/api/v1/orgs/acme/templates',
        event.request,
        event.locals.session as Session,
      );

    await handle({ event, resolve } as HandleArgs);

    const fetchMock = vi.mocked(fetch);
    const call = fetchMock.mock.calls[0];
    const outboundHeaders = new Headers(call[1]?.headers);
    expect(outboundHeaders.get('traceparent') ?? '').toMatch(TRACEPARENT_FORMAT);
  });

  it('carries the Authorization bearer header derived from the session alongside the continued traceparent', async () => {
    mockBackendFetchOk();
    const cookieValue = signSessionCookie(SESSION as unknown as Record<string, unknown>);
    const request = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_HEADER },
    });
    const event = buildFakeEvent(request, buildFakeCookies(cookieValue));
    const resolve = async (): Promise<Response> =>
      proxyToBackend(
        '/api/v1/orgs/acme/templates',
        event.request,
        event.locals.session as Session,
      );

    await handle({ event, resolve } as HandleArgs);

    const fetchMock = vi.mocked(fetch);
    const call = fetchMock.mock.calls[0];
    const outboundHeaders = new Headers(call[1]?.headers);
    expect(outboundHeaders.get('Authorization')).toBe(`Bearer ${SESSION.backendToken}`);
    expect(outboundHeaders.get('traceparent')?.split('-')[1]).toBe(INBOUND_TRACE_ID);
  });
});
