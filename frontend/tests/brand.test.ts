import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { DEFAULT_BRAND_CONFIG } from '../src/lib/types/brand';
import { buildBrandCssBlock, loadBrandConfig } from '../src/lib/brand';

vi.mock('node:fs/promises');

import { readFile } from 'node:fs/promises';

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

describe('loadBrandConfig()', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it('returns defaults when brand.toml is absent and no env vars are set', async () => {
    vi.mocked(readFile).mockRejectedValue(Object.assign(new Error('ENOENT'), { code: 'ENOENT' }));
    const config = await loadBrandConfig();
    expect(config).toEqual(DEFAULT_BRAND_CONFIG);
  });

  it('reads all six values from brand.toml', async () => {
    const toml = [
      'app_name           = "Acme Tools"',
      'logo_url           = "/static/logo.svg"',
      'logo_url_dark      = "/static/logo-dark.svg"',
      'logo_alt           = "Acme logo"',
      'primary_color      = "#d4451a"',
      'primary_color_dark = "#ff8c69"',
    ].join('\n');
    vi.mocked(readFile).mockResolvedValue(toml as unknown as Buffer);

    const config = await loadBrandConfig();

    expect(config.appName).toBe('Acme Tools');
    expect(config.logoUrl).toBe('/static/logo.svg');
    expect(config.logoUrlDark).toBe('/static/logo-dark.svg');
    expect(config.logoAlt).toBe('Acme logo');
    expect(config.primaryColor).toBe('#d4451a');
    expect(config.primaryColorDark).toBe('#ff8c69');
  });

  it('env vars take precedence over brand.toml values', async () => {
    vi.mocked(readFile).mockResolvedValue('app_name = "Acme"\nprimary_color = "#d4451a"\n' as unknown as Buffer);
    vi.stubEnv('BRAND_APP_NAME', 'Override Name');
    vi.stubEnv('BRAND_PRIMARY_COLOR', '#abcdef');

    const config = await loadBrandConfig();

    expect(config.appName).toBe('Override Name');
    expect(config.primaryColor).toBe('#abcdef');
  });

  it('env vars are applied when brand.toml is absent', async () => {
    vi.mocked(readFile).mockRejectedValue(Object.assign(new Error('ENOENT'), { code: 'ENOENT' }));
    vi.stubEnv('BRAND_APP_NAME', 'EnvOnly');
    vi.stubEnv('BRAND_PRIMARY_COLOR', '#123456');
    vi.stubEnv('BRAND_PRIMARY_COLOR_DARK', '#654321');

    const config = await loadBrandConfig();

    expect(config.appName).toBe('EnvOnly');
    expect(config.primaryColor).toBe('#123456');
    expect(config.primaryColorDark).toBe('#654321');
    // unset fields fall back to defaults
    expect(config.logoUrl).toBeNull();
    expect(config.logoAlt).toBe(DEFAULT_BRAND_CONFIG.logoAlt);
  });

  it('treats logo_url_dark without logo_url as both absent (renders wordmark)', async () => {
    vi.mocked(readFile).mockRejectedValue(Object.assign(new Error('ENOENT'), { code: 'ENOENT' }));
    vi.stubEnv('BRAND_LOGO_URL_DARK', '/static/dark-only.svg');

    const config = await loadBrandConfig();

    expect(config.logoUrl).toBeNull();
    expect(config.logoUrlDark).toBeNull();
  });

  it('treats logo_url_dark from toml without logo_url as both absent', async () => {
    vi.mocked(readFile).mockResolvedValue('logo_url_dark = "/static/dark.svg"\n' as unknown as Buffer);

    const config = await loadBrandConfig();

    expect(config.logoUrl).toBeNull();
    expect(config.logoUrlDark).toBeNull();
  });

  it('keeps logo_url_dark when logo_url is also set', async () => {
    vi.mocked(readFile).mockRejectedValue(Object.assign(new Error('ENOENT'), { code: 'ENOENT' }));
    vi.stubEnv('BRAND_LOGO_URL', '/static/logo.svg');
    vi.stubEnv('BRAND_LOGO_URL_DARK', '/static/logo-dark.svg');

    const config = await loadBrandConfig();

    expect(config.logoUrl).toBe('/static/logo.svg');
    expect(config.logoUrlDark).toBe('/static/logo-dark.svg');
  });

  it('produces a @media dark rule in the CSS block when primaryColorDark is configured', async () => {
    vi.mocked(readFile).mockRejectedValue(Object.assign(new Error('ENOENT'), { code: 'ENOENT' }));
    vi.stubEnv('BRAND_PRIMARY_COLOR', '#d4451a');
    vi.stubEnv('BRAND_PRIMARY_COLOR_DARK', '#ff8c69');

    const config = await loadBrandConfig();
    const css = buildBrandCssBlock(config);

    expect(css).toContain('--brand-primary: #d4451a');
    expect(css).toContain('@media (prefers-color-scheme: dark)');
    expect(css).toContain('--brand-primary: #ff8c69');
  });

  it('omits the @media dark rule when primaryColorDark is not configured', async () => {
    vi.mocked(readFile).mockRejectedValue(Object.assign(new Error('ENOENT'), { code: 'ENOENT' }));
    vi.stubEnv('BRAND_PRIMARY_COLOR', '#d4451a');

    const config = await loadBrandConfig();
    const css = buildBrandCssBlock(config);

    expect(css).toContain('--brand-primary: #d4451a');
    expect(css).not.toContain('@media (prefers-color-scheme: dark)');
  });

  it('continues with defaults when brand.toml contains a parse error', async () => {
    vi.mocked(readFile).mockResolvedValue('this is not valid toml =' as unknown as Buffer);

    // Should not throw; should fall through to defaults
    const config = await loadBrandConfig();
    expect(config.appName).toBe(DEFAULT_BRAND_CONFIG.appName);
  });
});
