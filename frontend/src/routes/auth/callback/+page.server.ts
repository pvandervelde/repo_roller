import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { PageServerLoad } from './$types';
import type { Session } from '$lib/types/session';

/**
 * OAuth callback handler.
 * Exchanges the authorization code for a token, retrieves GitHub identity,
 * sets the session cookie, and redirects to /create.
 * On any failure, redirects to /auth/denied with an appropriate reason.
 * SCR-002 / authentication.md
 */
export const load: PageServerLoad = async ({ url, cookies }) => {
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
    } catch {
        redirect(302, '/auth/denied?reason=identity_failure');
    }

    // Establish session cookie.
    const session: Session = { userLogin };
    cookies.set('session', JSON.stringify(session), {
        path: '/',
        httpOnly: true,
        secure: env.NODE_ENV === 'production',
        sameSite: 'lax',
    });

    redirect(302, '/create');
};
