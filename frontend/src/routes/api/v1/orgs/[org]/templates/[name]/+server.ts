import { json } from '@sveltejs/kit';
import { dev } from '$app/env';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

// Mock data uses the same shape as the real Rust backend:
//   variables is a Record<name, VariableDefinition> (camelCase fields)
//   tags is the full array of template tags (not a single category string)
const MOCK_TEMPLATE_DETAILS: Record<string, object> = {
  'basic-service': {
    name: 'basic-service',
    displayName: 'Basic Service',
    description: 'A minimal REST API service with health check and CI/CD pipeline.',
    tags: ['service', 'rest', 'api'],
    variables: {},
    configuration: {},
  },
  'data-pipeline': {
    name: 'data-pipeline',
    displayName: 'Data Pipeline',
    description: 'A batch data processing pipeline with scheduling and monitoring.',
    tags: ['data', 'pipeline', 'batch'],
    variables: {
      project_name: {
        description: 'The name of the data project (used in resource naming).',
        required: true,
      },
      schedule_cron: {
        description: 'Cron expression for the pipeline schedule (default: 2am daily).',
        required: false,
        default: '0 2 * * *',
      },
      team_slack_channel: {
        description: 'Slack channel for pipeline alerts (optional).',
        required: false,
      },
    },
    configuration: {},
  },
  'frontend-app': {
    name: 'frontend-app',
    displayName: 'Frontend Application',
    description: 'A modern single-page application with TypeScript and testing setup.',
    tags: ['frontend', 'typescript', 'spa'],
    variables: {
      app_display_name: {
        description: 'Human-readable display name shown in the browser tab and README.',
        required: true,
      },
    },
    configuration: {},
  },
  library: {
    name: 'library',
    displayName: 'Shared Library',
    description: 'A reusable library with semantic versioning and publishing pipeline.',
    tags: ['library', 'semver'],
    variables: {},
    configuration: {},
  },
};

export const GET: RequestHandler = ({ request, params, locals }) => {
  if (!dev) {
    return proxyToBackend(
      `/api/v1/orgs/${encodeURIComponent(params.org)}/templates/${encodeURIComponent(params.name)}`,
      request,
      locals.session,
    );
  }
  const detail = MOCK_TEMPLATE_DETAILS[params.name];
  if (!detail) {
    return json(
      { error: { code: 'NotFound', message: `Template '${params.name}' not found` } },
      { status: 404 },
    );
  }
  return new Promise((resolve) => setTimeout(() => resolve(json(detail)), 200));
};
