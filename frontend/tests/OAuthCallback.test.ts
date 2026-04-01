import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import OAuthCallbackPage from '../src/routes/auth/callback/+page.svelte';

const brandConfig = {
  appName: 'RepoRoller',
  logoUrl: null,
  logoUrlDark: null,
  logoAlt: 'RepoRoller logo',
  primaryColor: '#0969da',
  primaryColorDark: null,
};

const baseProps = {
  data: {
    brandConfig,
    session: null,
  },
};

describe('OAuth Callback page (SCR-002)', () => {
  it('renders the "Completing sign-in" heading', () => {
    render(OAuthCallbackPage, { props: baseProps });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Completing sign-in');
  });

  it('renders the status message', () => {
    render(OAuthCallbackPage, { props: baseProps });
    expect(screen.getByText("You'll be redirected in a moment.")).toBeInTheDocument();
  });

  it('renders a spinner with role="status" and aria-label', () => {
    render(OAuthCallbackPage, { props: baseProps });
    const spinner = screen.getByRole('status');
    expect(spinner).toBeInTheDocument();
    expect(spinner).toHaveAttribute('aria-label', 'Signing in');
  });

  it('wraps content in a BrandCard', () => {
    const { container } = render(OAuthCallbackPage, { props: baseProps });
    expect(container.querySelector('.brand-card__wrapper')).toBeTruthy();
  });
});
