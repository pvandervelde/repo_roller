import { json } from '@sveltejs/kit';
import { dev } from '$app/env';
import type { RequestHandler } from './$types';
import { proxyToBackend } from '$lib/proxy.server';

export const GET: RequestHandler = ({ request, params, locals, url }) => {
  if (!dev) {
    return proxyToBackend(
      `/api/v1/orgs/${encodeURIComponent(params.org)}/creation-capabilities?${url.searchParams.toString()}`,
      request,
      locals.session,
    );
  }
  return json({
    forkEnabled: true,
    sourceHistoryMirrorEnabled: true,
    sourceOnlyMirrorEnabled: true,
    allowedSourceOwners: ['upstream-org', 'some-vendor-org'],
    forkTemplate: 'fork-settings',
    sourceHistoryMirrorTemplate: 'mirror-history-settings',
    sourceOnlyMirrorTemplate: 'mirror-scaffolding',
  });
};
