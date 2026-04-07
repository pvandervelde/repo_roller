import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals }) => {
  return {
    userLogin: locals.session?.userLogin ?? null,
    organization: process.env.GITHUB_ORG ?? '',
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
