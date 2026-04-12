import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import BrandCard from '../src/lib/components/BrandCard.svelte';

const cardContent = createRawSnippet(() => ({
  render: () => '<h1>Sign in</h1>',
}));

const baseProps = {
  appName: 'RepoRoller',
  children: cardContent,
};

describe('BrandCard', () => {
  // -------------------------------------------------------------------------
  // Logo / wordmark — UX-ASSERT-026
  // -------------------------------------------------------------------------

  it('renders text wordmark when logoUrl is null', () => {
    render(BrandCard, { props: baseProps });
    expect(screen.queryByRole('img')).toBeNull();
    expect(screen.getAllByText('RepoRoller').length).toBeGreaterThan(0);
  });

  it('renders an <img> when logoUrl is set', () => {
    render(BrandCard, {
      props: { ...baseProps, logoUrl: 'https://example.com/logo.svg' },
    });
    expect(screen.getByRole('img')).toHaveAttribute('src', 'https://example.com/logo.svg');
  });

  it('logo alt defaults to "[appName] logo"', () => {
    render(BrandCard, {
      props: { ...baseProps, logoUrl: 'https://example.com/logo.svg' },
    });
    expect(screen.getByRole('img', { name: 'RepoRoller logo' })).toBeInTheDocument();
  });

  it('uses custom logoAlt when provided', () => {
    render(BrandCard, {
      props: {
        ...baseProps,
        logoUrl: 'https://example.com/logo.svg',
        logoAlt: 'Acme Inc.',
      },
    });
    expect(screen.getByRole('img', { name: 'Acme Inc.' })).toBeInTheDocument();
  });

  it('renders a <picture> when both logoUrl and logoUrlDark are set', () => {
    const { container } = render(BrandCard, {
      props: {
        ...baseProps,
        logoUrl: 'https://example.com/logo-light.svg',
        logoUrlDark: 'https://example.com/logo-dark.svg',
      },
    });
    expect(container.querySelector('picture')).toBeTruthy();
  });

  it('<picture> has a dark-mode <source> referencing logoUrlDark', () => {
    const { container } = render(BrandCard, {
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

  it('appName text is in DOM even when logo is shown (UX-ASSERT-026)', () => {
    const { container } = render(BrandCard, {
      props: { ...baseProps, logoUrl: 'https://example.com/logo.svg' },
    });
    expect(container.textContent).toContain('RepoRoller');
  });

  // -------------------------------------------------------------------------
  // Slot content
  // -------------------------------------------------------------------------

  it('renders slot content inside the card', () => {
    render(BrandCard, { props: baseProps });
    expect(screen.getByRole('heading', { name: 'Sign in' })).toBeInTheDocument();
  });
});
