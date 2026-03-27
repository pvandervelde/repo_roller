import { describe, it, expect, beforeEach } from 'vitest';
import { DEFAULT_BRAND_CONFIG } from '../src/lib/types/brand';
import { buildBrandCssBlock } from '../src/lib/brand';

describe('DEFAULT_BRAND_CONFIG', () => {
  it('has app name "RepoRoller"', () => {
    expect(DEFAULT_BRAND_CONFIG.appName).toBe('RepoRoller');
  });

  it('has null logo URLs by default', () => {
    expect(DEFAULT_BRAND_CONFIG.logoUrl).toBeNull();
    expect(DEFAULT_BRAND_CONFIG.logoUrlDark).toBeNull();
  });

  it('has the correct default primary colour (GitHub blue)', () => {
    expect(DEFAULT_BRAND_CONFIG.primaryColor).toBe('#0969da');
  });

  it('has null dark primary colour by default', () => {
    expect(DEFAULT_BRAND_CONFIG.primaryColorDark).toBeNull();
  });

  it('has logo alt text that references the app name', () => {
    expect(DEFAULT_BRAND_CONFIG.logoAlt).toContain('RepoRoller');
  });
});

describe('buildBrandCssBlock()', () => {
  beforeEach(() => {
    // Ensure env is clean for each test
  });

  it('returns a non-empty string', () => {
    const css = buildBrandCssBlock(DEFAULT_BRAND_CONFIG);
    expect(css.length).toBeGreaterThan(0);
  });

  it('emits a :root block containing --brand-primary with the configured light colour', () => {
    const css = buildBrandCssBlock({ ...DEFAULT_BRAND_CONFIG, primaryColor: '#ff0000' });
    expect(css).toContain(':root');
    expect(css).toContain('--brand-primary: #ff0000');
  });

  it('includes a @media (prefers-color-scheme: dark) rule when primaryColorDark is set', () => {
    const css = buildBrandCssBlock({ ...DEFAULT_BRAND_CONFIG, primaryColorDark: '#cc0000' });
    expect(css).toContain('@media (prefers-color-scheme: dark)');
    expect(css).toContain('--brand-primary: #cc0000');
  });

  it('omits the @media dark rule when primaryColorDark is null', () => {
    const css = buildBrandCssBlock({ ...DEFAULT_BRAND_CONFIG, primaryColorDark: null });
    expect(css).not.toContain('@media (prefers-color-scheme: dark)');
  });

  it('falls back to the default colour when an invalid hex value is supplied', () => {
    const css = buildBrandCssBlock({
      ...DEFAULT_BRAND_CONFIG,
      primaryColor: 'red; --evil: injected',
    });
    expect(css).not.toContain('evil');
    expect(css).toContain('--brand-primary: #0969da');
  });

  it('does not inject the dark colour when it contains invalid characters', () => {
    const css = buildBrandCssBlock({
      ...DEFAULT_BRAND_CONFIG,
      primaryColorDark: 'red; --evil: injected',
    });
    expect(css).not.toContain('evil');
    expect(css).not.toContain('@media (prefers-color-scheme: dark)');
  });
});
