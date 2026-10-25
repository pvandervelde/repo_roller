/**
 * Server-only utility for proxying API requests to the Rust backend.
 *
 * Used by all /api/v1/** SvelteKit server routes in production. In development
 * these routes return mock data instead; this module is only called when
 * BACKEND_API_URL is set.
 *
 * Environment variables:
 *   BACKEND_API_URL   Required. Base URL of the Rust API, e.g. http://api:3000
 *
 * MUST NOT be imported from client-side code.
 */
import { BACKEND_API_URL } from '$app/env/private';
import type { Session } from '$lib/types/session';
import { getCurrentTraceparent } from './observability/trace-context';

// Headers that must not be forwarded to the backend.
const HOP_BY_HOP = ['host', 'cookie', 'authorization', 'connection', 'transfer-encoding'];

/**
 * Forward a request to the Rust backend at the given path, returning the
 * backend response verbatim.
 *
 * @param path   Full API path including query string, e.g. `/api/v1/orgs/acme/templates`
 * @param request Original SvelteKit request (method, headers, body are forwarded)
 * @param session Active session; if null, returns 401 without calling the backend
 */
export async function proxyToBackend(
  path: string,
  request: Request,
  session: Session | null,
): Promise<Response> {
  if (!session) {
    return new Response(
      JSON.stringify({ error: { code: 'Unauthorized', message: 'Authentication required' } }),
      { status: 401, headers: { 'Content-Type': 'application/json' } },
    );
  }

  const backendUrl = BACKEND_API_URL?.replace(/\/$/, '');
  if (!backendUrl) {
    return new Response(
      JSON.stringify({
        error: { code: 'ServiceUnavailable', message: 'Backend API URL is not configured' },
      }),
      { status: 503, headers: { 'Content-Type': 'application/json' } },
    );
  }

  const forwardHeaders = new Headers();
  for (const [key, value] of request.headers.entries()) {
    if (!HOP_BY_HOP.includes(key.toLowerCase())) {
      forwardHeaders.set(key, value);
    }
  }
  forwardHeaders.set('Authorization', `Bearer ${session.backendToken}`);
  // Override (not forward) any inbound traceparent: this represents the
  // SvelteKit-server's OWN span in the browser -> SvelteKit -> Rust chain,
  // as a child of whatever trace context was received, rather than silently
  // relaying the browser's original span id straight through with no
  // SvelteKit-server hop visible in the trace.
  forwardHeaders.set('traceparent', getCurrentTraceparent());

  const hasBody = request.method !== 'GET' && request.method !== 'HEAD';
  const body = hasBody ? await request.arrayBuffer() : undefined;

  try {
    return await fetch(`${backendUrl}${path}`, {
      method: request.method,
      headers: forwardHeaders,
      body,
    });
  } catch {
    return new Response(
      JSON.stringify({
        error: { code: 'BadGateway', message: 'Failed to reach the backend API' },
      }),
      { status: 502, headers: { 'Content-Type': 'application/json' } },
    );
  }
}
