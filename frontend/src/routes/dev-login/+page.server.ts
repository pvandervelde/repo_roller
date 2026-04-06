import { redirect } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { PageServerLoad } from './$types';

/**
 * Dev-only shortcut: sets a fake session cookie directly, bypassing GitHub OAuth.
 * Only works in development mode (dev === true).
 */
export const load: PageServerLoad = async ({ cookies, url }) => {
    if (!dev) {
        redirect(302, '/sign-in');
    }

    const userLogin = url.searchParams.get('user') ?? 'dev-user';
    const userAvatarUrl = `https://avatars.githubusercontent.com/u/1?v=4`;

    cookies.set(
        'session',
        JSON.stringify({ userLogin, userAvatarUrl }),
        {
            path: '/',
            httpOnly: true,
            secure: false,
            sameSite: 'lax',
            maxAge: 60 * 60 * 24, // 1 day
        },
    );

    redirect(302, '/create');
};
