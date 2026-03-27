import { DEFAULT_BRAND_CONFIG, type BrandConfig } from './types/brand';

/**
 * Load brand configuration from environment variables, brand.toml, or built-in defaults.
 *
 * Priority (highest first):
 *  1. Environment variables: BRAND_APP_NAME, BRAND_LOGO_URL, BRAND_LOGO_URL_DARK,
 *     BRAND_LOGO_ALT, BRAND_PRIMARY_COLOR, BRAND_PRIMARY_COLOR_DARK
 *  2. brand.toml in the deployment working directory
 *  3. Built-in defaults
 *
 * Full implementation provided in task 13.2. This stub returns built-in defaults.
 */
export async function loadBrandConfig(): Promise<BrandConfig> {
  return { ...DEFAULT_BRAND_CONFIG };
}

/**
 * Build the server-rendered <style> block content that injects --brand-primary
 * as a CSS custom property, with an optional @media dark override.
 *
 * Only valid CSS hex colour values are interpolated to prevent CSS injection.
 */
export function buildBrandCssBlock(_config: BrandConfig): string {
  // TODO: implement in task 13.2
  return '';
}
