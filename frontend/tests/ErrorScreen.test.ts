import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ErrorPage from '../src/routes/error/+page.svelte';
import type { ErrorReason } from '../src/routes/error/+page.ts';

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

function makeProps(reason: ErrorReason, hasSession = true) {
    return {
        data: {
            brandConfig,
            session: hasSession ? session : null,
            reason,
        },
    };
}

describe('Error screen (SCR-006)', () => {
    // -------------------------------------------------------------------------
    // Generic error (default / unknown reason)
    // -------------------------------------------------------------------------

    it('renders "Something went wrong" h1 for generic reason', () => {
        render(ErrorPage, { props: makeProps('generic') });
        expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Something went wrong');
    });

    it('renders "Something went wrong" h1 when reason is generic (default after load normalization)', () => {
        // The load function normalizes unknown/missing params to 'generic'
        render(ErrorPage, { props: makeProps('generic') });
        expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Something went wrong');
    });

    it('renders "Something went wrong" h1 — same output regardless of which generic call', () => {
        render(ErrorPage, { props: makeProps('generic') });
        expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Something went wrong');
    });

    it('renders generic error message with role="alert"', () => {
        render(ErrorPage, { props: makeProps('generic') });
        const alert = screen.getByRole('alert');
        expect(alert).toHaveTextContent(
            'An unexpected error occurred. If this keeps happening, contact your platform team.',
        );
    });

    it('renders "Try again" button for generic error when session exists', () => {
        render(ErrorPage, { props: makeProps('generic', true) });
        const btn = screen.getByRole('link', { name: 'Try again' });
        expect(btn).toHaveAttribute('href', '/create');
    });

    it('renders "Try again" button pointing to /sign-in when no session', () => {
        render(ErrorPage, { props: makeProps('generic', false) });
        const btn = screen.getByRole('link', { name: 'Try again' });
        expect(btn).toHaveAttribute('href', '/sign-in');
    });

    it('does not render "Sign in" button for generic error', () => {
        render(ErrorPage, { props: makeProps('generic') });
        expect(screen.queryByRole('link', { name: 'Sign in' })).not.toBeInTheDocument();
    });

    it('does not show session-expired heading for generic error', () => {
        render(ErrorPage, { props: makeProps('generic') });
        expect(
            screen.queryByRole('heading', { name: 'Your session has expired' }),
        ).not.toBeInTheDocument();
    });

    // -------------------------------------------------------------------------
    // Session expired variant
    // -------------------------------------------------------------------------

    it('renders "Your session has expired" h1 for session_expired reason', () => {
        render(ErrorPage, { props: makeProps('session_expired') });
        expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent(
            'Your session has expired',
        );
    });

    it('renders session-expired message with role="alert"', () => {
        render(ErrorPage, { props: makeProps('session_expired') });
        const alert = screen.getByRole('alert');
        expect(alert).toHaveTextContent('Please sign in again to continue.');
    });

    it('renders "Sign in" button pointing to /sign-in for session_expired', () => {
        render(ErrorPage, { props: makeProps('session_expired') });
        const btn = screen.getByRole('link', { name: 'Sign in' });
        expect(btn).toHaveAttribute('href', '/sign-in');
    });

    it('does not render "Try again" button for session_expired', () => {
        render(ErrorPage, { props: makeProps('session_expired') });
        expect(screen.queryByRole('link', { name: 'Try again' })).not.toBeInTheDocument();
    });

    // -------------------------------------------------------------------------
    // Accessibility
    // -------------------------------------------------------------------------

    it('error message uses role="alert" to announce to screen readers', () => {
        render(ErrorPage, { props: makeProps('generic') });
        expect(screen.getByRole('alert')).toBeInTheDocument();
    });

    it('icon is hidden from assistive technology (aria-hidden)', () => {
        const { container } = render(ErrorPage, { props: makeProps('generic') });
        const icon = container.querySelector('.error-screen__icon');
        expect(icon).toHaveAttribute('aria-hidden', 'true');
    });
});
