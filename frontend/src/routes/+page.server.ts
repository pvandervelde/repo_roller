import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// Root route: redirect to /create when authenticated, otherwise /sign-in.
export const load: PageServerLoad = async ({ locals }) => {
  if (locals.session) {
    redirect(302, '/create');
  }
  redirect(302, '/sign-in');
};
