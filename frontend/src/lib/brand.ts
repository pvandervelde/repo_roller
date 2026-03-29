import { readFile } from 'node:fs/promises';
import { parse } from 'smol-toml';
import { DEFAULT_BRAND_CONFIG, type BrandConfig } from './types/brand';

/**
 * Only valid CSS hex colour strings are safe to interpolate into a <style> block.
 * Accepts 3-, 4-, 6-, and 8-digit hex notation (e.g. #rgb, #rgba, #rrggbb, #rrggbbaa).
 */
const HEX_COLOR_RE = /^#[0-9a-fA-F]{3,8}$/;

function sanitizeHex(value: string, fallback: string): string {
  return HEX_COLOR_RE.test(value) ? value : fallback;
}

/** Shape of fields accepted in brand.toml. */
interface TomlBrandConfig {
  app_name?: unknown;
  logo_url?: unknown;
  logo_url_dark?: unknown;
  logo_alt?: unknown;
  primary_color?: unknown;
  primary_color_dark?: unknown;
}

/**
 * Load brand configuration from environment variables, brand.toml, or built-in defaults.
 *
 * Priority (highest first):
 *  1. Environment variables: BRAND_APP_NAME, BRAND_LOGO_URL, BRAND_LOGO_URL_DARK,
 *     BRAND_LOGO_ALT, BRAND_PRIMARY_COLOR, BRAND_PRIMARY_COLOR_DARK
 *  2. brand.toml in the server's working directory
 *  3. Built-in defaults
 *
 * Security: brand.toml must NOT be placed inside static/ — it is a server-side file only.
 * Rule: logo_url_dark without logo_url is not valid; both are treated as absent.
 */
export async function loadBrandConfig(): Promise<BrandConfig> {
  const config: BrandConfig = { ...DEFAULT_BRAND_CONFIG };

  // Layer 2: brand.toml
  try {
    const raw = await readFile('brand.toml', 'utf-8');
    const toml = parse(raw) as TomlBrandConfig;

    if (typeof toml.app_name === 'string') config.appName = toml.app_name;
    if (typeof toml.logo_url === 'string') config.logoUrl = toml.logo_url;
    if (typeof toml.logo_url_dark === 'string') config.logoUrlDark = toml.logo_url_dark;
    if (typeof toml.logo_alt === 'string') config.logoAlt = toml.logo_alt;
    if (typeof toml.primary_color === 'string') config.primaryColor = toml.primary_color;
    if (typeof toml.primary_color_dark === 'string')
      config.primaryColorDark = toml.primary_color_dark;
  } catch (err) {
    const code = (err as NodeJS.ErrnoException).code;
    if (code !== 'ENOENT') {
      // Parse errors or unexpected I/O errors — log and continue with whatever we have
      console.warn('[brand] Failed to read brand.toml:', err);
    }
  }

  // Layer 1: environment variables (highest priority)
  const env = process.env;
  if (env.BRAND_APP_NAME) config.appName = env.BRAND_APP_NAME;
  if (env.BRAND_LOGO_URL) config.logoUrl = env.BRAND_LOGO_URL;
  if (env.BRAND_LOGO_URL_DARK) config.logoUrlDark = env.BRAND_LOGO_URL_DARK;
  if (env.BRAND_LOGO_ALT) config.logoAlt = env.BRAND_LOGO_ALT;
  if (env.BRAND_PRIMARY_COLOR) config.primaryColor = env.BRAND_PRIMARY_COLOR;
  if (env.BRAND_PRIMARY_COLOR_DARK) config.primaryColorDark = env.BRAND_PRIMARY_COLOR_DARK;

  // Enforce: logo_url_dark without logo_url is not a supported configuration
  if (!config.logoUrl && config.logoUrlDark) {
    config.logoUrlDark = null;
  }

  return config;
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
