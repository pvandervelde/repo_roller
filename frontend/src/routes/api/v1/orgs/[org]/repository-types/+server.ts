import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

export const GET: RequestHandler = ({ request, params, locals }) => {
  if (!dev) {
    return proxyToBackend(
      `/api/v1/orgs/${encodeURIComponent(params.org)}/repository-types`,
      request,
      locals.session,
    );
  }
  return json({
    types: [
      { name: 'service', description: 'Backend service or REST API' },
      { name: 'frontend', description: 'Frontend web application' },
      { name: 'library', description: 'Reusable shared library' },
      { name: 'data-pipeline', description: 'Batch data processing pipeline' },
    ]
  });
};
