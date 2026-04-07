import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';

const MOCK_TEMPLATE_DETAILS: Record<string, object> = {
  'basic-service': {
    name: 'basic-service',
    description: 'A minimal REST API service with health check and CI/CD pipeline.',
    tags: ['service', 'rest', 'api'],
    repository_type: { policy: 'preferable', type_name: 'service' },
    variables: [],
  },
  'data-pipeline': {
    name: 'data-pipeline',
    description: 'A batch data processing pipeline with scheduling and monitoring.',
    tags: ['data', 'pipeline', 'batch'],
    repository_type: { policy: 'optional', type_name: null },
    variables: [
      {
        name: 'project_name',
        required: true,
        default_value: null,
        description: 'The name of the data project (used in resource naming).',
      },
      {
        name: 'schedule_cron',
        required: false,
        default_value: '0 2 * * *',
        description: 'Cron expression for the pipeline schedule (default: 2am daily).',
      },
      {
        name: 'team_slack_channel',
        required: false,
        default_value: null,
        description: 'Slack channel for pipeline alerts (optional).',
      },
    ],
  },
  'frontend-app': {
    name: 'frontend-app',
    description: 'A modern single-page application with TypeScript and testing setup.',
    tags: ['frontend', 'spa', 'typescript'],
    repository_type: { policy: 'fixed', type_name: 'frontend' },
    variables: [
      {
        name: 'app_display_name',
        required: true,
        default_value: null,
        description: 'Human-readable display name shown in the browser tab and README.',
      },
    ],
  },
  library: {
    name: 'library',
    description: 'A reusable library with semantic versioning and publishing pipeline.',
    tags: ['library', 'npm', 'versioning'],
    repository_type: { policy: 'optional', type_name: null },
    variables: [],
  },
};

export const GET: RequestHandler = ({ params }) => {
  if (!dev) {
    return new Response('Not Found', { status: 404 });
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
