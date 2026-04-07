import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';

const RESERVED = ['admin', 'api', 'www', 'test-exists'];

export const POST: RequestHandler = async ({ request }) => {
  if (!dev) {
    return new Response('Not Found', { status: 404 });
  }
  const body = (await request.json()) as { name?: string };
  const name = (body.name ?? '').trim();

  if (!name) {
    return json({
      valid: false,
      name,
      errors: [{ field: 'name', message: 'Name is required', constraint: 'required' }],
    });
  }
  if (!/^[a-z0-9]([a-z0-9-]{0,98}[a-z0-9])?$/.test(name)) {
    return json({
      valid: false,
      name,
      errors: [
        {
          field: 'name',
          message: 'Name must be lowercase alphanumeric with hyphens',
          constraint: 'format',
        },
      ],
    });
  }
  if (RESERVED.includes(name)) {
    return json({
      valid: false,
      name,
      errors: [
        { field: 'name', message: 'Repository name is already taken', constraint: 'unique' },
      ],
    });
  }
  // Simulate async availability check
  return new Promise((resolve) => setTimeout(() => resolve(json({ valid: true, name })), 600));
};
