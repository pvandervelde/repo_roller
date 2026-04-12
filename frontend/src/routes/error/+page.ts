import type { PageLoad } from './$types';

export type ErrorReason = 'session_expired' | 'generic';

export const load: PageLoad = ({ url }) => {
  const raw = url.searchParams.get('reason') ?? '';
  const reason: ErrorReason = raw === 'session_expired' ? 'session_expired' : 'generic';
  return { reason };
};
