import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

export const POST: RequestHandler = async ({ request, locals }) => {
  if (!dev) {
    return proxyToBackend('/api/v1/repositories', request, locals.session);
  }
  const body = (await request.json()) as {
    organization?: string;
    name?: string;
    template?: string;
    visibility?: string;
    team?: string;
    repository_type?: string;
    variables?: Record<string, string>;
  };

  const org = body.organization ?? 'demo-org';
  const name = body.name ?? 'my-repo';

  // Simulate a short creation delay
  await new Promise((r) => setTimeout(r, 1500));

  const now = new Date().toISOString();
  return json(
    {
      repository: {
        name,
        fullName: `${org}/${name}`,
        url: `https://github.com/${org}/${name}`,
        visibility: body.visibility ?? 'private',
      },
      appliedConfiguration: {
        template: body.template,
        visibility: body.visibility ?? 'private',
        team: body.team ?? null,
        repository_type: body.repository_type ?? null,
      },
      createdAt: now,
    },
    { status: 201 },
  );
};
