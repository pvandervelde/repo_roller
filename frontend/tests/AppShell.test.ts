import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import AppShell from '../src/lib/components/AppShell.svelte';

const slotContent = createRawSnippet(() => ({
  render: () => '<p>Page content</p>',
}));

const baseProps = {
  appName: 'RepoRoller',
  userLogin: 'octocat',
  children: slotContent,
};

describe('AppShell', () => {
  // -------------------------------------------------------------------------
  // Landmarks
  // -------------------------------------------------------------------------

  it('renders a <header> landmark', () => {
    render(AppShell, { props: baseProps });
    expect(screen.getByRole('banner')).toBeInTheDocument();
  });

  it('renders a <main> landmark', () => {
    render(AppShell, { props: baseProps });
    expect(screen.getByRole('main')).toBeInTheDocument();
  });

  it('renders slot content inside the main landmark', () => {
    render(AppShell, { props: baseProps });
    expect(screen.getByRole('main')).toHaveTextContent('Page content');
  });

  // -------------------------------------------------------------------------
  // Logo / wordmark — UX-ASSERT-026
  // -------------------------------------------------------------------------

  it('renders text wordmark when logoUrl is null', () => {
    render(AppShell, { props: { ...baseProps, logoUrl: null } });
    expect(screen.queryByRole('img')).toBeNull();
    expect(screen.getAllByText('RepoRoller').length).toBeGreaterThan(0);
  });

  it('renders an <img> when logoUrl is set (no dark variant)', () => {
    render(AppShell, {
      props: { ...baseProps, logoUrl: 'https://example.com/logo.svg' },
    });
    const img = screen.getByRole('img');
    expect(img).toHaveAttribute('src', 'https://example.com/logo.svg');
  });

  it('logo alt defaults to "[appName] logo"', () => {
    render(AppShell, {
      props: { ...baseProps, logoUrl: 'https://example.com/logo.svg' },
    });
    expect(screen.getByRole('img', { name: 'RepoRoller logo' })).toBeInTheDocument();
  });

  it('uses custom logoAlt when provided', () => {
    render(AppShell, {
      props: {
        ...baseProps,
        logoUrl: 'https://example.com/logo.svg',
        logoAlt: 'Acme logo',
      },
    });
    expect(screen.getByRole('img', { name: 'Acme logo' })).toBeInTheDocument();
  });

  it('renders a <picture> when both logoUrl and logoUrlDark are set', () => {
    const { container } = render(AppShell, {
      props: {
        ...baseProps,
        logoUrl: 'https://example.com/logo-light.svg',
        logoUrlDark: 'https://example.com/logo-dark.svg',
      },
    });
    expect(container.querySelector('picture')).toBeTruthy();
  });

  it('<picture> has a dark-mode <source> referencing logoUrlDark', () => {
    const { container } = render(AppShell, {
      props: {
        ...baseProps,
        logoUrl: 'https://example.com/logo-light.svg',
        logoUrlDark: 'https://example.com/logo-dark.svg',
      },
    });
    const source = container.querySelector('source[media*="dark"]');
    expect(source).toBeTruthy();
    expect(source?.getAttribute('srcset')).toBe('https://example.com/logo-dark.svg');
  });

  it('appName text is always present in DOM even when logo is shown (UX-ASSERT-026)', () => {
    const { container } = render(AppShell, {
      props: { ...baseProps, logoUrl: 'https://example.com/logo.svg' },
    });
    expect(container.textContent).toContain('RepoRoller');
  });

  // -------------------------------------------------------------------------
  // UserBadge integration
  // -------------------------------------------------------------------------

  it('renders the authenticated user login via UserBadge', () => {
    render(AppShell, { props: { ...baseProps, userLogin: 'octocat' } });
    expect(screen.getByText('octocat')).toBeInTheDocument();
  });

  it('forwards onsignOut when "Sign out" is clicked', async () => {
    const onsignOut = vi.fn();
    render(AppShell, { props: { ...baseProps, onsignOut } });
    await fireEvent.click(screen.getByRole('button'));
    await fireEvent.click(screen.getByRole('menuitem', { name: 'Sign out' }));
    expect(onsignOut).toHaveBeenCalledOnce();
  });
});
