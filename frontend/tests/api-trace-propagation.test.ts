/**
 * Verifies that `src/lib/api/client.ts`'s internal `apiFetch()` helper
 * attaches a well-formed W3C `traceparent` header to every outgoing request,
 * satisfying the acceptance criterion: "Frontend API calls to the backend
 * include a trace-context header (traceparent) that the backend recognizes
 * and continues."
 *
 * Exercised indirectly through the public functions in client.ts (GET and
 * POST) since `apiFetch` itself is not exported — this also proves the
 * header is attached consistently across call sites, not just one.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { listTemplates, createRepository, validateRepositoryName } from '../src/lib/api/client';
import { TRACEPARENT_FORMAT } from '../src/lib/observability/trace-context';

vi.mock('../src/lib/observability/trace-context', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/lib/observability/trace-context')>();
  return {
    ...actual,
    getCurrentTraceparent: vi.fn(actual.getCurrentTraceparent),
  };
});

import { getCurrentTraceparent } from '../src/lib/observability/trace-context';

const FIXED_HEADER = '00-11111111111111111111111111111111-2222222222222222-01';

function mockFetchOk(body: unknown = {}): void {
  // A fresh Response per invocation — a Response body can only be read once,
  // and some tests here call the API client function more than once.
  vi.stubGlobal(
    'fetch',
    vi.fn().mockImplementation(
      async () =>
        new Response(JSON.stringify(body), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
    ),
  );
}

beforeEach(() => {
  vi.unstubAllGlobals();
  vi.mocked(getCurrentTraceparent).mockReset();
});

describe('apiFetch — traceparent header attachment (GET)', () => {
  it('attaches the value returned by getCurrentTraceparent() as the traceparent header', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(FIXED_HEADER);
    mockFetchOk({ templates: [] });

    await listTemplates('myorg');

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(FIXED_HEADER);
  });

  it('calls getCurrentTraceparent() at least once per request', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(FIXED_HEADER);
    mockFetchOk({ templates: [] });

    await listTemplates('myorg');

    expect(getCurrentTraceparent).toHaveBeenCalled();
  });
});

describe('apiFetch — traceparent header attachment (POST with JSON body)', () => {
  it('attaches traceparent on createRepository() without losing Content-Type', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(FIXED_HEADER);
    mockFetchOk({
      repository: { name: 'r', fullName: 'o/r', url: 'https://x', visibility: 'private' },
      appliedConfiguration: {},
      createdAt: '2026-01-01T00:00:00Z',
    });

    await createRepository({ organization: 'o', name: 'r', template: 't' });

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(FIXED_HEADER);
    expect(headers.get('Content-Type')).toBe('application/json');
  });

  it('attaches traceparent on validateRepositoryName()', async () => {
    vi.mocked(getCurrentTraceparent).mockReturnValue(FIXED_HEADER);
    mockFetchOk({ valid: true, available: true });

    await validateRepositoryName('o', 'name');

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent')).toBe(FIXED_HEADER);
  });
});

describe('apiFetch — real (unmocked) traceparent is well-formed end-to-end', () => {
  it('sends a header matching TRACEPARENT_FORMAT when the real trace-context module runs', async () => {
    const { getCurrentTraceparent: real } = await vi.importActual<
      typeof import('../src/lib/observability/trace-context')
    >('../src/lib/observability/trace-context');
    vi.mocked(getCurrentTraceparent).mockImplementation(real);
    mockFetchOk({ templates: [] });

    await listTemplates('myorg');

    const call = vi.mocked(fetch).mock.calls[0];
    const headers = new Headers(call[1]?.headers);
    expect(headers.get('traceparent') ?? '').toMatch(TRACEPARENT_FORMAT);
  });
});

describe('apiFetch — distinct requests get distinct headers when no active span exists (adversarial)', () => {
  it('does not send the exact same traceparent on two independent calls using the real module', async () => {
    const { getCurrentTraceparent: real } = await vi.importActual<
      typeof import('../src/lib/observability/trace-context')
    >('../src/lib/observability/trace-context');
    vi.mocked(getCurrentTraceparent).mockImplementation(real);
    mockFetchOk({ templates: [] });

    await listTemplates('myorg');
    await listTemplates('myorg');

    const firstHeaders = new Headers(vi.mocked(fetch).mock.calls[0][1]?.headers);
    const secondHeaders = new Headers(vi.mocked(fetch).mock.calls[1][1]?.headers);
    expect(firstHeaders.get('traceparent')).not.toBe(secondHeaders.get('traceparent'));
  });
});
