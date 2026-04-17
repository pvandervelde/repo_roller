import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';
import type { Session } from '$lib/types/session';
import { parseSessionCookie } from '$lib/session.server';

/** Routes that require an authenticated session. */
const PROTECTED_PREFIXES = ['/create'];

export const handle: Handle = async ({ event, resolve }) => {
  // Attempt to restore session from the signed HTTP-only cookie.
  const sessionCookie = event.cookies.get('session');
  if (sessionCookie) {
    const parsed = parseSessionCookie<unknown>(sessionCookie);
    if (
      parsed !== null &&
      typeof parsed === 'object' &&
      'userLogin' in (parsed as object) &&
      typeof (parsed as Session).userLogin === 'string' &&
      typeof (parsed as Session).backendToken === 'string' &&
      (parsed as Session).backendToken.length > 0
    ) {
      event.locals.session = parsed as Session;
    } else {
      // Invalid or forged cookie — clear it.
      event.locals.session = null;
      event.cookies.delete('session', { path: '/' });
    }
  } else {
    event.locals.session = null;
  }

  // Auth guard: redirect unauthenticated users away from protected routes.
  const isProtected = PROTECTED_PREFIXES.some((prefix) => event.url.pathname.startsWith(prefix));
  if (isProtected && !event.locals.session) {
    redirect(302, '/sign-in');
  }

  return resolve(event);
};
