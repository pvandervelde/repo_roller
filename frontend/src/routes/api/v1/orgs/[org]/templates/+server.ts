import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';

const MOCK_TEMPLATES = [
    {
        name: 'basic-service',
        description: 'A minimal REST API service with health check and CI/CD pipeline.',
        tags: ['service', 'rest', 'api'],
        repository_type: { policy: 'preferable', type_name: 'service' },
    },
    {
        name: 'data-pipeline',
        description: 'A batch data processing pipeline with scheduling and monitoring.',
        tags: ['data', 'pipeline', 'batch'],
        repository_type: { policy: 'optional', type_name: null },
    },
    {
        name: 'frontend-app',
        description: 'A modern single-page application with TypeScript and testing setup.',
        tags: ['frontend', 'spa', 'typescript'],
        repository_type: { policy: 'fixed', type_name: 'frontend' },
    },
    {
        name: 'library',
        description: 'A reusable library with semantic versioning and publishing pipeline.',
        tags: ['library', 'npm', 'versioning'],
        repository_type: { policy: 'optional', type_name: null },
    },
];

export const GET: RequestHandler = ({ params }) => {
    if (!dev) {
        return new Response('Not Found', { status: 404 });
    }
    // Small artificial delay so loading states are visible
    return new Promise((resolve) =>
        setTimeout(
            () => resolve(json({ templates: MOCK_TEMPLATES })),
            400,
        ),
    );
};
