import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { PageServerLoad } from './$types';

/**
 * Build the GitHub OAuth authorization URL with CSRF state token.
 * SCR-001 / authentication.md
 */
export const load: PageServerLoad = async ({ cookies, locals }) => {
  // Already authenticated — skip sign-in.
  if (locals.session) {
    redirect(302, '/create');
  }

  // Generate a random CSRF state token.
  const stateBytes = new Uint8Array(16);
  crypto.getRandomValues(stateBytes);
  const state = Array.from(stateBytes, (b) => b.toString(16).padStart(2, '0')).join('');

  // Store state in a short-lived cookie for verification on callback.
  cookies.set('oauth_state', state, {
    path: '/auth/callback',
    httpOnly: true,
    secure: env.NODE_ENV === 'production',
    sameSite: 'lax',
    maxAge: 600, // 10 minutes
  });

  const clientId = env.GITHUB_CLIENT_ID ?? '';
  const params = new URLSearchParams({
    client_id: clientId,
    scope: 'read:user read:org',
    state,
  });

  return {
    githubAuthUrl: `https://github.com/login/oauth/authorize?${params.toString()}`,
  };
};
