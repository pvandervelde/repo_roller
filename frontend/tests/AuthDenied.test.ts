import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import AuthDeniedPage from '../src/routes/auth/denied/+page.svelte';
import type { DenialReason } from '../src/routes/auth/denied/+page.server';

const brandConfig = {
  appName: 'RepoRoller',
  logoUrl: null,
  logoUrlDark: null,
  logoAlt: 'RepoRoller logo',
  primaryColor: '#0969da',
  primaryColorDark: null,
};

function makeProps(reason: DenialReason) {
  return {
    data: {
      brandConfig,
      session: null,
      reason,
    },
  };
}

describe('Access Denied page (SCR-003)', () => {
  // -------------------------------------------------------------------------
  // oauth_error variant
  // -------------------------------------------------------------------------

  it('renders "Sign-in could not be completed" heading for oauth_error', () => {
    render(AuthDeniedPage, { props: makeProps('oauth_error') });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Sign-in could not be completed',
    );
  });

  it('renders the network error message for oauth_error', () => {
    render(AuthDeniedPage, { props: makeProps('oauth_error') });
    expect(
      screen.getByText('There was a problem connecting to GitHub. This is usually temporary.'),
    ).toBeInTheDocument();
  });

  it('renders "Try again" button for oauth_error', () => {
    render(AuthDeniedPage, { props: makeProps('oauth_error') });
    expect(screen.getByRole('link', { name: 'Try again' })).toHaveAttribute('href', '/sign-in');
  });

  it('renders same copy for network_error as oauth_error', () => {
    render(AuthDeniedPage, { props: makeProps('network_error') });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Sign-in could not be completed',
    );
  });

  it('renders same copy for identity_failure as oauth_error', () => {
    render(AuthDeniedPage, { props: makeProps('identity_failure') });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'Sign-in could not be completed',
    );
  });

  // -------------------------------------------------------------------------
  // access_denied variant
  // -------------------------------------------------------------------------

  it('renders "GitHub authorization was cancelled" heading for access_denied', () => {
    render(AuthDeniedPage, { props: makeProps('access_denied') });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
      'GitHub authorization was cancelled',
    );
  });

  it('renders the access_denied message for access_denied', () => {
    render(AuthDeniedPage, { props: makeProps('access_denied') });
    expect(screen.getByText(/needs permission to read your GitHub identity/i)).toBeInTheDocument();
  });

  it('renders "Try again" button for access_denied', () => {
    render(AuthDeniedPage, { props: makeProps('access_denied') });
    expect(screen.getByRole('link', { name: 'Try again' })).toHaveAttribute('href', '/sign-in');
  });

  // -------------------------------------------------------------------------
  // BrandCard shell
  // -------------------------------------------------------------------------

  it('wraps content in a BrandCard', () => {
    const { container } = render(AuthDeniedPage, { props: makeProps('oauth_error') });
    expect(container.querySelector('.brand-card__wrapper')).toBeTruthy();
  });

  // -------------------------------------------------------------------------
  // Accessibility
  // -------------------------------------------------------------------------

  it('error message has role="alert" for screen reader announcement', () => {
    render(AuthDeniedPage, { props: makeProps('oauth_error') });
    expect(screen.getByRole('alert')).toBeInTheDocument();
  });
});
