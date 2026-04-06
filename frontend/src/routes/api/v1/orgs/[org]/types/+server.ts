import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = () => {
    if (!dev) {
        return new Response('Not Found', { status: 404 });
    }
    return json({ types: ['service', 'frontend', 'library', 'data-pipeline'] });
};
