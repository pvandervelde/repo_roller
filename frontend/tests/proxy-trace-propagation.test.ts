/**
 * Verifies that `src/lib/proxy.server.ts`'s `proxyToBackend()` attaches a
 * well-formed W3C `traceparent` header to its outbound fetch call to the
 * real Rust backend, satisfying the second hop of the acceptance criterion:
 * "Frontend API calls to the backend include a trace-context header
 * (traceparent) that the backend recognizes and continues."
 *
 * Architecture note: `proxyToBackend` already forwards ALL non-hop-by-hop
 * headers from the incoming SvelteKit request, including whatever
 * `traceparent` the browser's own `apiFetch()` call attached when it hit the
 * SvelteKit server. That generic forwarding is necessary but not sufficient:
 * per the task brief's explicitly-sanctioned manual-propagation fallback,
 * `proxyToBackend` must call `getCurrentTraceparent()` itself and set the
 * result explicitly (the same way it already does for the `Authorization`
 * header, which is also first generically copied and then overridden) —
 * this represents the SvelteKit-server's OWN span in the browser → SvelteKit
 * → Rust chain, rather than silently relaying the browser's original span id
 * straight through to Rust with no SvelteKit-server hop visible in the
 * trace. The tests below specifically distinguish "explicitly set by this
 * module" from "passively forwarded unchanged" by giving the incoming
 * request a DIFFERENT traceparent than the one `getCurrentTraceparent()` is
 * mocked to return, and asserting the outbound header matches the latter.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import fc from 'fast-check';
import { TRACEPARENT_FORMAT } from '../src/lib/observability/trace-context';
import type { Session } from '../src/lib/types/session';

// `$app/env/private` is a SvelteKit virtual module. Under Vite's dev/test
// ("serve") command it is generated as a static snapshot object baked in at
// module-resolution time (see `@sveltejs/kit/src/core/env.js`'s
// `create_env_modules`), NOT a live view of `process.env` — so
// `vi.stubEnv('BACKEND_API_URL', ...)` in a `beforeEach` has no effect on it
// (confirmed experimentally while writing this suite: the env var was always
// seen as unset by `proxy.server.ts`, causing every call to short-circuit to
// 503 regardless of what was stubbed). Mocking the virtual module directly is
// the deterministic way to control it from a test.
const { mockEnv } = vi.hoisted(() => ({
  mockEnv: { BACKEND_API_URL: 'http://backend.internal:3000' } as Record<
    string,
    string | undefined
  >,
}));

vi.mock('$app/env/private', () => ({
  get BACKEND_API_URL() {
    return mockEnv.BACKEND_API_URL;
  },
}));

vi.mock('../src/lib/observability/trace-context', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/lib/observability/trace-context')>();
  return {
    ...actual,
    getCurrentTraceparent: vi.fn(actual.getCurrentTraceparent),
  };
});

const { proxyToBackend } = await import('../src/lib/proxy.server');
const { getCurrentTraceparent } = await import('../src/lib/observability/trace-context');

const SESSION: Session = {
  userLogin: 'jdoe',
  userAvatarUrl: null,
  backendToken: 'test-jwt',
};

const OWN_HEADER = '00-33333333333333333333333333333333-4444444444444444-01';
const INBOUND_BROWSER_HEADER = '00-55555555555555555555555555555555-6666666666666666-01';

function mockBackendFetchOk(): void {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockImplementation(async () => new Response('{}', { status: 200 })),
  );
}

beforeEach(() => {
  vi.unstubAllGlobals();
  mockEnv.BACKEND_API_URL = 'http://backend.internal:3000';
  vi.mocked(getCurrentTraceparent).mockReset();
});

describe('proxyToBackend — traceparent header attachment', () => {
  it('attaches the value returned by getCurrentTraceparent() to the outbound fetch', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/orgs/acme/templates');

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(OWN_HEADER);
  });

  it('overrides an inbound traceparent header with its own, rather than passively forwarding it', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: INBOUND_BROWSER_HEADER },
    });

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(OWN_HEADER);
    expect(headers.get('traceparent')).not.toBe(INBOUND_BROWSER_HEADER);
  });

  it('still attaches a well-formed traceparent when the inbound request has none at all', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/orgs/acme/templates');

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent') ?? '').toMatch(TRACEPARENT_FORMAT);
  });

  it('preserves the Authorization bearer header alongside the new traceparent header', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/orgs/acme/templates');

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(OWN_HEADER);
    expect(headers.get('Authorization')).toBe('Bearer test-jwt');
  });

  it('preserves a caller-supplied Content-Type header alongside the new traceparent header', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/repositories', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'x' }),
    });

    await proxyToBackend('/api/v1/repositories', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(OWN_HEADER);
    expect(headers.get('Content-Type')).toBe('application/json');
  });
});

describe('proxyToBackend — real (unmocked) traceparent is well-formed end-to-end', () => {
  it('sends a header matching TRACEPARENT_FORMAT when the real trace-context module runs', async () => {
    const { getCurrentTraceparent: real } = await vi.importActual<
      typeof import('../src/lib/observability/trace-context')
    >('../src/lib/observability/trace-context');
    vi.mocked(getCurrentTraceparent).mockImplementation(real);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/orgs/acme/templates');

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent') ?? '').toMatch(TRACEPARENT_FORMAT);
  });
});

describe('proxyToBackend — malformed/duplicated inbound header shapes (adversarial, QA audit)', () => {
  // The Fetch/Headers spec combines multiple entries for the same header
  // name into a SINGLE comma-joined value at Request-construction time (so
  // `request.headers.entries()` never actually yields two separate
  // `traceparent` entries) — but the override in `proxyToBackend` is
  // unconditional (`forwardHeaders.set('traceparent', getCurrentTraceparent())`
  // always runs after the generic-forwarding loop, with no "is this already
  // a valid traceparent?" check), so this is safe either way. This test
  // pins that down explicitly rather than relying on it being an accidental
  // consequence of the Headers spec.
  it('still overrides with its own value when the inbound request has multiple traceparent-shaped header entries', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    const req = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: [
        ['traceparent', INBOUND_BROWSER_HEADER],
        ['traceparent', '00-77777777777777777777777777777777-8888888888888888-01'],
      ],
    });
    // Sanity check on the test's own premise: the Headers spec already
    // combined the two entries into one comma-joined (and therefore
    // W3C-malformed) value before proxyToBackend ever sees it.
    expect(req.headers.get('traceparent')).toContain(',');

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(OWN_HEADER);
  });

  it('still overrides with its own value when the inbound traceparent is almost-but-not-quite valid W3C format', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(OWN_HEADER);
    mockBackendFetchOk();
    // One hex digit short in the trace-id segment (31 chars, not 32) — looks
    // superficially plausible but fails TRACEPARENT_FORMAT.
    const almostValid = '00-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bbbbbbbbbbbbbbbb-01';
    const req = new Request('http://localhost/api/v1/orgs/acme/templates', {
      headers: { traceparent: almostValid },
    });

    await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(OWN_HEADER);
    expect(headers.get('traceparent')).not.toBe(almostValid);
  });
});

describe('proxyToBackend — property: outbound header is always well-formed regardless of inbound header (adversarial)', () => {
  // Restricted to characters that are always legal in an HTTP header value
  // (hex digits and dashes), so the generator produces realistic
  // "malformed traceparent" noise — wrong length, wrong grouping, all-zero
  // ids, empty string — without ever tripping the Headers API's own
  // "Invalid header value" validation, which would test the wrong thing.
  const malformedHeaderArb = fc.oneof(
    fc.constant(undefined),
    fc.constant(''),
    fc.constant('not-a-traceparent'),
    fc.constant(`00-${'0'.repeat(32)}-${'0'.repeat(16)}-00`), // all-zero, invalid per W3C spec
    fc
      .array(fc.constantFrom(...'0123456789abcdef-'.split('')), { maxLength: 60 })
      .map((chars) => chars.join('')),
  );

  it('is well-formed for arbitrary (possibly malformed or absent) inbound traceparent header values', async () => {
    const { getCurrentTraceparent: real } = await vi.importActual<
      typeof import('../src/lib/observability/trace-context')
    >('../src/lib/observability/trace-context');
    vi.mocked(getCurrentTraceparent).mockImplementation(real);

    await fc.assert(
      fc.asyncProperty(malformedHeaderArb, async (inboundValue) => {
        mockBackendFetchOk();
        const headersInit: Record<string, string> =
          inboundValue !== undefined ? { traceparent: inboundValue } : {};
        const req = new Request('http://localhost/api/v1/orgs/acme/templates', {
          headers: headersInit,
        });

        const res = await proxyToBackend('/api/v1/orgs/acme/templates', req, SESSION);
        expect(res.status).toBe(200);

        const call = vi.mocked(fetch).mock.calls.at(-1);
        const outboundHeaders = new Headers(call?.[1]?.headers);
        expect(outboundHeaders.get('traceparent') ?? '').toMatch(TRACEPARENT_FORMAT);
      }),
      { numRuns: 25 },
    );
  });
});
