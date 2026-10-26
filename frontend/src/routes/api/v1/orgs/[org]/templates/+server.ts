import { json } from '@sveltejs/kit';
import { dev } from '$app/env';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

const MOCK_TEMPLATES = [
  {
    name: 'basic-service',
    displayName: 'Basic Service',
    description: 'A minimal REST API service with health check and CI/CD pipeline.',
    tags: ['service', 'rest', 'api'],
    variables: [],
  },
  {
    name: 'data-pipeline',
    displayName: 'Data Pipeline',
    description: 'A batch data processing pipeline with scheduling and monitoring.',
    tags: ['data', 'pipeline', 'batch'],
    variables: ['project_name', 'schedule_cron', 'team_slack_channel'],
  },
  {
    name: 'frontend-app',
    displayName: 'Frontend Application',
    description: 'A modern single-page application with TypeScript and testing setup.',
    tags: ['frontend', 'typescript', 'spa'],
    variables: ['app_display_name'],
  },
  {
    name: 'library',
    displayName: 'Shared Library',
    description: 'A reusable library with semantic versioning and publishing pipeline.',
    tags: ['library', 'semver'],
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
