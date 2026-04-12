import { loadBrandConfig } from '$lib/brand.server';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals }) => {
  const brandConfig = await loadBrandConfig();
  return {
    brandConfig,
    session: locals.session,
  };
};
