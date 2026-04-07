import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SuccessPage from '../src/routes/create/success/+page.svelte';

const brandConfig = {
  appName: 'RepoRoller',
  logoUrl: null,
  logoUrlDark: null,
  logoAlt: 'RepoRoller logo',
  primaryColor: '#0969da',
  primaryColorDark: null,
};

const session = {
  userLogin: 'alice',
  userAvatarUrl: 'https://example.com/avatar.png',
};

function makeProps(repo: string) {
  return {
    data: {
      brandConfig,
      session,
      repo,
    },
  };
}

describe('Repository Created page (SCR-005)', () => {
  // -------------------------------------------------------------------------
  // Valid repo param — happy path (UX-ASSERT-022)
  // -------------------------------------------------------------------------

  it('renders "Repository created!" h1 when repo param is valid', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository created!');
  });

  it('shows the full repository name in a code element (UX-ASSERT-022)', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    expect(screen.getByText('test-org/my-repo')).toBeInTheDocument();
  });

  it('renders the GitHub URL link pointing to the correct URL (UX-ASSERT-022)', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    const link = screen.getByRole('link', {
      name: /View test-org\/my-repo on GitHub \(opens in new tab\)/i,
    });
    expect(link).toHaveAttribute('href', 'https://github.com/test-org/my-repo');
  });

  it('renders "View repository on GitHub" button when param is valid', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    // The button text is the accessible name (no aria-label override)
    const btn = screen.getByRole('link', { name: 'View repository on GitHub ↗' });
    expect(btn).toBeInTheDocument();
  });

  it('"View repository on GitHub" button links to GitHub URL (UX-ASSERT-022)', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    const btn = screen.getByRole('link', { name: 'View repository on GitHub ↗' });
    expect(btn).toHaveAttribute('href', 'https://github.com/test-org/my-repo');
  });

  it('"View repository on GitHub" button opens in a new tab', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    const btn = screen.getByRole('link', { name: 'View repository on GitHub ↗' });
    expect(btn).toHaveAttribute('target', '_blank');
    expect(btn).toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('does not show the invalid message when repo param is valid', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    expect(
      screen.queryByText('Your repository was created. Check your GitHub organization to find it.'),
    ).not.toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // "Create another repository" link (UX-ASSERT-023)
  // -------------------------------------------------------------------------

  it('renders "Create another repository" link pointing to /create (valid param)', () => {
    render(SuccessPage, { props: makeProps('test-org/my-repo') });
    const link = screen.getByRole('link', { name: 'Create another repository' });
    expect(link).toHaveAttribute('href', '/create');
  });

  it('renders "Create another repository" link even when repo param is invalid', () => {
    render(SuccessPage, { props: makeProps('') });
    const link = screen.getByRole('link', { name: 'Create another repository' });
    expect(link).toHaveAttribute('href', '/create');
  });

  // -------------------------------------------------------------------------
  // Invalid / missing repo param
  // -------------------------------------------------------------------------

  it('shows the fallback info message when repo param is empty', () => {
    render(SuccessPage, { props: makeProps('') });
    expect(
      screen.getByText('Your repository was created. Check your GitHub organization to find it.'),
    ).toBeInTheDocument();
  });

  it('fallback message has role="status" for assistive technology', () => {
    render(SuccessPage, { props: makeProps('') });
    const msg = screen.getByRole('status');
    expect(msg).toHaveTextContent(
      'Your repository was created. Check your GitHub organization to find it.',
    );
  });

  it('does not render the h1 heading when repo param is invalid', () => {
    render(SuccessPage, { props: makeProps('not-valid') });
    expect(screen.queryByRole('heading', { level: 1 })).not.toBeInTheDocument();
  });

  it('does not render "View repository on GitHub" button when repo param is invalid', () => {
    render(SuccessPage, { props: makeProps('not-valid') });
    expect(
      screen.queryByRole('link', { name: 'View repository on GitHub ↗' }),
    ).not.toBeInTheDocument();
  });

  it('shows fallback message when repo has no slash (treated as invalid)', () => {
    render(SuccessPage, { props: makeProps('noslash') });
    expect(
      screen.getByText('Your repository was created. Check your GitHub organization to find it.'),
    ).toBeInTheDocument();
  });
});
