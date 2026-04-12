import { DEFAULT_BRAND_CONFIG, type BrandConfig } from './types/brand';

const HEX_COLOR_RE = /^#[0-9a-fA-F]{3,8}$/;

function sanitizeHex(value: string, fallback: string): string {
  return HEX_COLOR_RE.test(value) ? value : fallback;
}

/**
 * Build the content of a server-rendered <style> block that injects the
 * --brand-primary CSS custom property, with an optional @media dark override.
 *
 * Only valid CSS hex colour values are interpolated to prevent CSS injection.
 *
 * @example
 * // Light-only:
 * buildBrandCssBlock({ primaryColor: '#0969da', primaryColorDark: null, ... })
 * // → ":root { --brand-primary: #0969da; }"
 *
 * @example
 * // With dark override:
 * buildBrandCssBlock({ primaryColor: '#0969da', primaryColorDark: '#58a6ff', ... })
 * // → ":root { --brand-primary: #0969da; } @media (prefers-color-scheme: dark) { :root { --brand-primary: #58a6ff; } }"
 */
export function buildBrandCssBlock(config: BrandConfig): string {
  const lightColor = sanitizeHex(config.primaryColor, DEFAULT_BRAND_CONFIG.primaryColor);
  const light = `:root { --brand-primary: ${lightColor}; }`;

  if (config.primaryColorDark && HEX_COLOR_RE.test(config.primaryColorDark)) {
    const dark = `@media (prefers-color-scheme: dark) { :root { --brand-primary: ${config.primaryColorDark}; } }`;
    return `${light} ${dark}`;
  }

  return light;
}
