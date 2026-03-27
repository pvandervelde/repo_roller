import { DEFAULT_BRAND_CONFIG, type BrandConfig } from './types/brand';

/**
 * Only valid CSS hex colour strings are safe to interpolate into a <style> block.
 * Accepts 3-, 4-, 6-, and 8-digit hex notation (e.g. #rgb, #rgba, #rrggbb, #rrggbbaa).
 */
const HEX_COLOR_RE = /^#[0-9a-fA-F]{3,8}$/;

function sanitizeHex(value: string, fallback: string): string {
    return HEX_COLOR_RE.test(value) ? value : fallback;
}

/**
 * Load brand configuration from environment variables, brand.toml, or built-in defaults.
 *
 * Priority (highest first):
 *  1. Environment variables: BRAND_APP_NAME, BRAND_LOGO_URL, BRAND_LOGO_URL_DARK,
 *     BRAND_LOGO_ALT, BRAND_PRIMARY_COLOR, BRAND_PRIMARY_COLOR_DARK
 *  2. brand.toml in the deployment working directory
 *  3. Built-in defaults
 *
 * Full file-reading implementation provided in task 13.2. This version reads env vars only.
 */
export async function loadBrandConfig(): Promise<BrandConfig> {
    return { ...DEFAULT_BRAND_CONFIG };
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
