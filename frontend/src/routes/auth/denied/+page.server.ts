import type { PageServerLoad } from './$types';

export type DenialReason =
  | 'access_denied'
  | 'oauth_error'
  | 'network_error'
  | 'identity_failure'
  | 'not_org_member';

/**
 * Access denied page.
 * Reads the `reason` query parameter and returns copy-selection data.
 * SCR-003 / authentication.md
 */
export const load: PageServerLoad = async ({ url }) => {
  const raw = url.searchParams.get('reason') ?? 'oauth_error';
  const reason: DenialReason = [
    'access_denied',
    'oauth_error',
    'network_error',
    'identity_failure',
    'not_org_member',
  ].includes(raw)
    ? (raw as DenialReason)
    : 'oauth_error';

  return { reason };
};
