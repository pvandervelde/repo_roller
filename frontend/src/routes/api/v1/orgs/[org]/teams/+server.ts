import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

const MOCK_TEAMS = [
    { slug: 'platform-team', name: 'Platform Team' },
    { slug: 'backend-team', name: 'Backend Team' },
    { slug: 'frontend-team', name: 'Frontend Team' },
];

export const GET: RequestHandler = ({ request, params, locals }) => {
    if (!dev) {
        return proxyToBackend(
            `/api/v1/orgs/${encodeURIComponent(params.org)}/teams`,
            request,
            locals.session,
        );
    }
    return json({ teams: MOCK_TEAMS });
};
