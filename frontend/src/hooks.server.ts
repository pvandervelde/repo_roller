import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';
import type { Session } from '$lib/types/session';

/** Routes that require an authenticated session. */
const PROTECTED_PREFIXES = ['/create'];

export const handle: Handle = async ({ event, resolve }) => {
    // Attempt to restore session from the HTTP-only cookie.
    const sessionCookie = event.cookies.get('session');
    if (sessionCookie) {
        try {
            const parsed: unknown = JSON.parse(sessionCookie);
            if (
                parsed !== null &&
                typeof parsed === 'object' &&
                'userLogin' in parsed &&
                typeof (parsed as Session).userLogin === 'string'
            ) {
                event.locals.session = parsed as Session;
            } else {
                event.locals.session = null;
                event.cookies.delete('session', { path: '/' });
            }
        } catch {
            // Malformed cookie — clear it and treat as unauthenticated.
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
