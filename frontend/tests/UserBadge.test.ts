import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import UserBadge from '../src/lib/components/UserBadge.svelte';

// ---------------------------------------------------------------------------
// #286 issue 5: username unreadable in dark mode.
//
// `.user-badge__toggle` has no explicit `color`, so the username text
// defaults to the browser's black button text color against the dark-mode
// `--color-surface` background. Several other rules also hardcode
// non-token colors (#fff, #e5e7eb, #374151, #f3f4f6, rgba(0,0,0,0.05)) that
// don't adapt to the `prefers-color-scheme: dark` media query defined in
// `src/app.css`.
//
// jsdom's CSS engine does not reliably compute cross-stylesheet values for
// scoped Svelte component styles (and `getComputedStyle` in this project's
// test setup is not configured to resolve CSS custom properties), so a
// computed-style assertion would be unreliable here. Following the existing
// precedent in `no-console-calls.test.ts` (AST/source-based regression
// guard for a similar "no raw X left in source" acceptance criterion), these
// tests read the component's raw source text directly and assert that the
// specific hardcoded color literals are gone from their originating rules
// and have been replaced with the appropriate `var(--color-*)` tokens.
// ---------------------------------------------------------------------------

/** Reads `UserBadge.svelte`'s raw source text. */
function readUserBadgeSource(): string {
  return readFileSync(
    join(__dirname, '..', 'src', 'lib', 'components', 'UserBadge.svelte'),
    'utf-8',
  );
}

/** Extracts the text between the component's `<style>` and `</style>` tags. */
function extractStyleBlock(source: string): string {
  const match = source.match(/<style>([\s\S]*?)<\/style>/);
  if (!match) {
    throw new Error('Expected to find a <style> block in UserBadge.svelte');
  }
  return match[1];
}

/**
 * Extracts the declaration body of a single top-level CSS rule by exact
 * selector text (e.g. `.user-badge__toggle` vs. `.user-badge__toggle:hover`
 * are distinct selectors and must not match each other's rule body).
 */
function extractRule(styleBlock: string, selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const match = styleBlock.match(new RegExp(`${escaped}\\s*\\{([^}]*)\\}`));
  if (!match) {
    throw new Error(`Expected to find a rule for selector "${selector}" in UserBadge.svelte`);
  }
  return match[1];
}

describe('UserBadge', () => {
  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  it('renders the GitHub username', () => {
    render(UserBadge, { props: { login: 'octocat' } });
    expect(screen.getByText('octocat')).toBeInTheDocument();
  });

  it('renders an avatar image when avatarUrl is provided', () => {
    render(UserBadge, {
      props: { login: 'octocat', avatarUrl: 'https://avatars.example.com/octocat' },
    });
    const img = screen.getByRole('img');
    expect(img).toHaveAttribute('src', 'https://avatars.example.com/octocat');
  });

  it('sets avatar alt text to "[login]\'s GitHub avatar"', () => {
    render(UserBadge, {
      props: { login: 'octocat', avatarUrl: 'https://avatars.example.com/octocat' },
    });
    expect(screen.getByAltText("octocat's GitHub avatar")).toBeInTheDocument();
  });

  it('renders initials fallback when avatarUrl is null', () => {
    render(UserBadge, { props: { login: 'octocat', avatarUrl: null } });
    expect(screen.queryByRole('img')).toBeNull();
    // Initials or username text still present
    expect(screen.getByText('octocat')).toBeInTheDocument();
  });

  it('renders initials fallback when avatarUrl is omitted', () => {
    render(UserBadge, { props: { login: 'jsmith' } });
    expect(screen.queryByRole('img')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Dropdown toggle button — UX-ASSERT-004
  // -------------------------------------------------------------------------

  it('renders a dropdown toggle button', () => {
    render(UserBadge, { props: { login: 'octocat' } });
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('toggle button has aria-haspopup="true"', () => {
    render(UserBadge, { props: { login: 'octocat' } });
    expect(screen.getByRole('button')).toHaveAttribute('aria-haspopup', 'true');
  });

  it('toggle button starts with aria-expanded="false"', () => {
    render(UserBadge, { props: { login: 'octocat' } });
    expect(screen.getByRole('button')).toHaveAttribute('aria-expanded', 'false');
  });

  it('sets aria-expanded="true" when the dropdown is open', async () => {
    render(UserBadge, { props: { login: 'octocat' } });
    await fireEvent.click(screen.getByRole('button'));
    expect(screen.getByRole('button')).toHaveAttribute('aria-expanded', 'true');
  });

  it('toggles dropdown closed again on second click', async () => {
    render(UserBadge, { props: { login: 'octocat' } });
    await fireEvent.click(screen.getByRole('button'));
    await fireEvent.click(screen.getByRole('button'));
    expect(screen.getByRole('button')).toHaveAttribute('aria-expanded', 'false');
  });

  // -------------------------------------------------------------------------
  // "Sign out" menu item — copy.md: "Sign out"
  // -------------------------------------------------------------------------

  it('shows "Sign out" menu item when dropdown is open', async () => {
    render(UserBadge, { props: { login: 'octocat' } });
    await fireEvent.click(screen.getByRole('button'));
    expect(screen.getByRole('menuitem', { name: 'Sign out' })).toBeInTheDocument();
  });

  it('does not show "Sign out" before dropdown is opened', () => {
    render(UserBadge, { props: { login: 'octocat' } });
    expect(screen.queryByRole('menuitem', { name: 'Sign out' })).toBeNull();
  });

  // -------------------------------------------------------------------------
  // signOut event — UX-ASSERT-004
  // -------------------------------------------------------------------------

  it('calls onsignOut callback when "Sign out" is clicked', async () => {
    const onsignOut = vi.fn();
    render(UserBadge, { props: { login: 'octocat', onsignOut } });
    await fireEvent.click(screen.getByRole('button'));
    await fireEvent.click(screen.getByRole('menuitem', { name: 'Sign out' }));
    expect(onsignOut).toHaveBeenCalledOnce();
  });

  // -------------------------------------------------------------------------
  // Dark-mode color regression (#286 issue 5)
  // -------------------------------------------------------------------------

  describe('dark-mode theming', () => {
    const styleBlock = extractStyleBlock(readUserBadgeSource());

    it('`.user-badge__toggle` declares an explicit color using the --color-text token (regression: username defaults to unreadable black text in dark mode)', () => {
      const rule = extractRule(styleBlock, '.user-badge__toggle');
      expect(rule).toMatch(/color:\s*var\(--color-text\)/);
    });

    it('`.user-badge__menu` background no longer hardcodes #fff and uses the --color-surface token', () => {
      const rule = extractRule(styleBlock, '.user-badge__menu');
      expect(rule.toLowerCase()).not.toContain('#fff');
      expect(rule).toMatch(/background-color:\s*var\(--color-surface\)/);
    });

    it('`.user-badge__menu` border no longer hardcodes #e5e7eb and uses the --color-border token', () => {
      const rule = extractRule(styleBlock, '.user-badge__menu');
      expect(rule.toLowerCase()).not.toContain('e5e7eb');
      expect(rule).toMatch(/border:[^;]*var\(--color-border\)/);
    });

    it('`.user-badge__menu-item` text color no longer hardcodes #374151 and uses the --color-text token', () => {
      const rule = extractRule(styleBlock, '.user-badge__menu-item');
      expect(rule.toLowerCase()).not.toContain('374151');
      expect(rule).toMatch(/color:\s*var\(--color-text\)/);
    });

    it('`.user-badge__menu-item:hover` no longer hardcodes the #f3f4f6 background and uses a themed token', () => {
      const rule = extractRule(styleBlock, '.user-badge__menu-item:hover');
      expect(rule.toLowerCase()).not.toContain('f3f4f6');
      expect(rule).toMatch(/var\(--color-[\w-]+\)/);
    });

    it('`.user-badge__toggle:hover` no longer hardcodes rgba(0, 0, 0, 0.05) and uses a themed token', () => {
      const rule = extractRule(styleBlock, '.user-badge__toggle:hover');
      expect(rule.replace(/\s+/g, '')).not.toContain('rgba(0,0,0,0.05)');
      expect(rule).toMatch(/var\(--color-[\w-]+\)/);
    });

    it('contains none of the five legacy hardcoded color literals anywhere in the <style> block', () => {
      const normalized = styleBlock.replace(/\s+/g, '').toLowerCase();
      const legacyLiterals = ['#fff', '#e5e7eb', '#374151', '#f3f4f6', 'rgba(0,0,0,0.05)'];
      const offenders = legacyLiterals.filter((literal) => normalized.includes(literal));
      expect(offenders).toEqual([]);
    });
  });
});
