import { redirect, error } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
  const organization = process.env.GITHUB_ORG;
  if (!organization) {
    // GITHUB_ORG is required for all API calls. Fail fast with a clear message
    // rather than making invalid requests to /api/v1/orgs//...
    error(503, 'GITHUB_ORG environment variable is not set. Contact your platform administrator.');
  }

  return {
    userLogin: locals.session?.userLogin ?? null,
    organization,
  };
};

/**
 * Sign-out action.
 * Destroys the session cookie and redirects to /sign-in.
 * UX-ASSERT-004
 */
export const actions: Actions = {
  signOut: async ({ cookies }) => {
    cookies.delete('session', { path: '/' });
    redirect(302, '/sign-in');
  },
};
