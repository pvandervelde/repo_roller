import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

const RESERVED = ['admin', 'api', 'www', 'test-exists'];

export const POST: RequestHandler = async ({ request, locals }) => {
  if (!dev) {
    return proxyToBackend('/api/v1/repositories/validate-name', request, locals.session);
  }
  const body = (await request.json()) as { name?: string };
  const name = (body.name ?? '').trim();

  if (!name) {
    return json({ valid: false, available: false, messages: ['Name is required'] });
  }
  if (!/^[a-z0-9]([a-z0-9-]{0,98}[a-z0-9])?$/.test(name)) {
    return json({
      valid: false,
      available: false,
      messages: ['Name must be lowercase alphanumeric with hyphens'],
    });
  }
  if (RESERVED.includes(name)) {
    return json({ valid: true, available: false, messages: ['Repository name is already taken'] });
  }
  // Simulate async availability check
  return new Promise((resolve) =>
    setTimeout(() => resolve(json({ valid: true, available: true })), 600),
  );
};
