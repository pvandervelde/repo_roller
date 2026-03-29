import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import UserBadge from '../src/lib/components/UserBadge.svelte';

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
});
