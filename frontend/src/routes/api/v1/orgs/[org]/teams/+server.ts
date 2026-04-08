import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';

const MOCK_TEAMS = [
    { slug: 'platform-team', name: 'Platform Team' },
    { slug: 'backend-team', name: 'Backend Team' },
    { slug: 'frontend-team', name: 'Frontend Team' },
];

export const GET: RequestHandler = () => {
    if (!dev) {
        return new Response('Not Found', { status: 404 });
    }
    return json({ teams: MOCK_TEAMS });
};
