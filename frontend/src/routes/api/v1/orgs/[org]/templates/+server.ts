import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

const MOCK_TEMPLATES = [
  {
    name: 'basic-service',
    description: 'A minimal REST API service with health check and CI/CD pipeline.',
    category: 'service',
    variables: [],
  },
  {
    name: 'data-pipeline',
    description: 'A batch data processing pipeline with scheduling and monitoring.',
    category: 'data',
    variables: ['project_name', 'schedule_cron', 'team_slack_channel'],
  },
  {
    name: 'frontend-app',
    description: 'A modern single-page application with TypeScript and testing setup.',
    category: 'frontend',
    variables: ['app_display_name'],
  },
  {
    name: 'library',
    description: 'A reusable library with semantic versioning and publishing pipeline.',
    category: 'library',
    variables: [],
  },
];

export const GET: RequestHandler = ({ request, params, locals }) => {
  if (!dev) {
    return proxyToBackend(
      `/api/v1/orgs/${encodeURIComponent(params.org)}/templates`,
      request,
      locals.session,
    );
  }
  // Small artificial delay so loading states are visible
  return new Promise((resolve) =>
    setTimeout(() => resolve(json({ templates: MOCK_TEMPLATES })), 400),
  );
};
