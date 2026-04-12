/**
 * Server-only utility for proxying API requests to the Rust backend.
 *
 * Used by all /api/v1/** SvelteKit server routes in production. In development
 * these routes return mock data instead; this module is only called when
 * BACKEND_API_URL is set.
 *
 * Environment variables:
 *   BACKEND_API_URL   Required. Base URL of the Rust API, e.g. http://api:3000
 *   BACKEND_API_TOKEN Optional. Bearer token for backend authentication.
 *
 * MUST NOT be imported from client-side code.
 */
import { env } from '$env/dynamic/private';

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
  session: unknown,
): Promise<Response> {
  if (!session) {
    return new Response(
      JSON.stringify({ error: { code: 'Unauthorized', message: 'Authentication required' } }),
      { status: 401, headers: { 'Content-Type': 'application/json' } },
    );
  }

  const backendUrl = env.BACKEND_API_URL?.replace(/\/$/, '');
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
  const token = env.BACKEND_API_TOKEN;
  if (token) {
    forwardHeaders.set('Authorization', `Bearer ${token}`);
  }

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
