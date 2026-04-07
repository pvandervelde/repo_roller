import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
  if (!dev) {
    return new Response('Not Found', { status: 404 });
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
        full_name: `${org}/${name}`,
        url: `https://github.com/${org}/${name}`,
        visibility: body.visibility ?? 'private',
        repository_type: body.repository_type ?? null,
        created_at: now,
      },
      configuration: {
        applied_settings: {
          template: body.template,
          visibility: body.visibility ?? 'private',
          team: body.team ?? null,
          repository_type: body.repository_type ?? null,
        },
        sources: {
          template: 'template-registry',
          visibility: 'user-input',
          team: body.team ? 'user-input' : 'default',
        },
      },
    },
    { status: 201 },
  );
};
