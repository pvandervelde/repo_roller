import { redirect } from '@sveltejs/kit';
import { dev } from '$app/env';
import type { PageServerLoad } from './$types';
import type { Session } from '$lib/types/session';
import { signSessionCookie } from '$lib/session.server';

/**
 * Dev-only shortcut: sets a fake session cookie directly, bypassing GitHub OAuth.
 * Only works in development mode (dev === true).
 */
export const load: PageServerLoad = async ({ cookies, url }) => {
  if (!dev) {
    redirect(302, '/sign-in');
  }

  const userLogin = url.searchParams.get('user') ?? 'dev-user';
  // Use the GitHub avatar URL pattern for the given username rather than
  // hardcoding a specific user ID.
  const userAvatarUrl = `https://avatars.githubusercontent.com/${encodeURIComponent(userLogin)}?v=4`;

  // backendToken is a placeholder, not a real credential: dev-login bypasses
  // the real OAuth + token-exchange flow entirely. It must be non-empty to
  // satisfy hooks.server.ts's session guard, but in normal dev usage (no
  // BACKEND_API_URL) all API routes return mock data and proxy.server.ts is
  // never called, so this value is never sent to a backend. If you need to
  // proxy real API calls in development, use the full OAuth flow instead of
  // this shortcut.
  const session: Session = { userLogin, userAvatarUrl, backendToken: 'dev-login-placeholder' };

  cookies.set('session', signSessionCookie(session), {
    path: '/',
    httpOnly: true,
    secure: false,
    sameSite: 'lax',
    maxAge: 60 * 60 * 24, // 1 day
  });

  redirect(302, '/create');
};
