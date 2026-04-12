/**
 * Brand configuration for deployment-time white-labelling.
 *
 * Values are loaded from environment variables, brand.toml, or built-in defaults
 * at server startup. See docs/spec/ux/branding.md and ADR-008.
 */
export interface BrandConfig {
  /** Configured application name shown in titles and headings. */
  appName: string;

  /**
   * Light-mode logo image URL. If null, appName is rendered as a text wordmark.
   * Must be a relative path (/static/...) or an absolute https:// URL.
   */
  logoUrl: string | null;

  /**
   * Dark-mode logo image URL. Used in a <picture> element alongside logoUrl when set.
   * Ignored if logoUrl is null.
   */
  logoUrlDark: string | null;

  /** Alt text for the logo image. Defaults to "[appName] logo". */
  logoAlt: string;

  /** Primary brand colour for light mode. CSS hex string (e.g. "#0969da"). */
  primaryColor: string;

  /**
   * Primary brand colour for dark mode. When set, a @media (prefers-color-scheme: dark)
   * override is injected. Falls back to primaryColor when null.
   */
  primaryColorDark: string | null;
}

/** Built-in defaults applied when no brand configuration is present. */
export const DEFAULT_BRAND_CONFIG: BrandConfig = {
  appName: 'RepoRoller',
  logoUrl: null,
  logoUrlDark: null,
  logoAlt: 'RepoRoller logo',
  primaryColor: '#0969da',
  primaryColorDark: null,
};
