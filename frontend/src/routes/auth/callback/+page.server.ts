import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { PageServerLoad } from './$types';
import type { Session } from '$lib/types/session';
import { signSessionCookie } from '$lib/session.server';

/**
 * OAuth callback handler.
 * Exchanges the authorization code for a token, retrieves GitHub identity,
 * sets the session cookie, and redirects to /create.
 * On any failure, redirects to /auth/denied with an appropriate reason.
 * SCR-002 / authentication.md
 *
 * This load function always redirects and never returns data to the page.
 * The page renders only very briefly while the server processes the callback.
 */
export const load: PageServerLoad = async ({ url, cookies }): Promise<Record<string, never>> => {
  const code = url.searchParams.get('code');
  const state = url.searchParams.get('state');
  const error = url.searchParams.get('error');

  // GitHub sent an error / user cancelled.
  if (error === 'access_denied') {
    redirect(302, '/auth/denied?reason=access_denied');
  }
  if (error) {
    redirect(302, '/auth/denied?reason=oauth_error');
  }

  // Verify CSRF state.
  const expectedState = cookies.get('oauth_state');
  cookies.delete('oauth_state', { path: '/auth/callback' });
  if (!state || state !== expectedState) {
    redirect(302, '/auth/denied?reason=oauth_error');
  }

  if (!code) {
    redirect(302, '/auth/denied?reason=oauth_error');
  }

  // Exchange code for access token.
  let accessToken: string;
  try {
    const tokenRes = await fetch('https://github.com/login/oauth/access_token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
      body: JSON.stringify({
        client_id: env.GITHUB_CLIENT_ID ?? '',
        client_secret: env.GITHUB_CLIENT_SECRET ?? '',
        code,
      }),
    });
    if (!tokenRes.ok) {
      redirect(302, '/auth/denied?reason=oauth_error');
    }
    const tokenData = (await tokenRes.json()) as Record<string, unknown>;
    if (typeof tokenData['access_token'] !== 'string' || !tokenData['access_token']) {
      redirect(302, '/auth/denied?reason=oauth_error');
    }
    accessToken = tokenData['access_token'];
  } catch {
    redirect(302, '/auth/denied?reason=network_error');
  }

  // Retrieve GitHub user identity.
  let userLogin: string;
  let userAvatarUrl: string | null = null;
  try {
    const userRes = await fetch('https://api.github.com/user', {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        Accept: 'application/vnd.github+json',
      },
    });
    if (!userRes.ok) {
      redirect(302, '/auth/denied?reason=identity_failure');
    }
    const userData = (await userRes.json()) as Record<string, unknown>;
    if (typeof userData['login'] !== 'string' || !userData['login']) {
      redirect(302, '/auth/denied?reason=identity_failure');
    }
    userLogin = userData['login'];
    userAvatarUrl = typeof userData['avatar_url'] === 'string' ? userData['avatar_url'] : null;
  } catch {
    redirect(302, '/auth/denied?reason=identity_failure');
  }

  // Exchange the GitHub OAuth token for a short-lived backend JWT.
  // The backend validates the GitHub token once and returns a signed JWT
  // (ADR-009) that is used as the Bearer on all subsequent API requests.
  let backendToken: string;
  const backendUrl = env.BACKEND_API_URL?.replace(/\/$/, '');
  if (!backendUrl) {
    redirect(302, '/auth/denied?reason=configuration_error');
  }
  try {
    const exchangeRes = await fetch(`${backendUrl}/api/v1/auth/token`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    });
    if (!exchangeRes.ok) {
      redirect(302, '/auth/denied?reason=token_exchange_failed');
    }
    const exchangeData = (await exchangeRes.json()) as Record<string, unknown>;
    if (typeof exchangeData['token'] !== 'string' || !exchangeData['token']) {
      redirect(302, '/auth/denied?reason=token_exchange_failed');
    }
    backendToken = exchangeData['token'];
  } catch {
    redirect(302, '/auth/denied?reason=network_error');
  }

  // Establish signed session cookie. The HMAC prevents cookie forgery.
  const session: Session = { userLogin, userAvatarUrl, backendToken };
  cookies.set('session', signSessionCookie(session), {
    path: '/',
    httpOnly: true,
    secure: env.NODE_ENV === 'production',
    sameSite: 'lax',
    // 8-hour session lifetime. Re-authenticate after this window.
    maxAge: 60 * 60 * 8,
  });

  redirect(302, '/create');
  // TypeScript requires a return type that matches PageServerLoad.
  // The redirect above always fires; this line is unreachable.
  return {};
};
