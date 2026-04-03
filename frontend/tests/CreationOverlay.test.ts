import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import CreationOverlay from '../src/lib/components/CreationOverlay.svelte';

describe('CreationOverlay (CMP-011)', () => {
    // -------------------------------------------------------------------------
    // Visibility
    // -------------------------------------------------------------------------

    it('renders nothing when visible is false', () => {
        render(CreationOverlay, { props: { visible: false } });
        expect(screen.queryByRole('dialog')).toBeNull();
    });

    it('renders the overlay when visible is true', () => {
        render(CreationOverlay, { props: { visible: true } });
        expect(screen.getByRole('dialog')).toBeInTheDocument();
    });

    // -------------------------------------------------------------------------
    // Content — copy.md strings
    // -------------------------------------------------------------------------

    it('renders the overlay heading text', () => {
        render(CreationOverlay, { props: { visible: true } });
        expect(screen.getByText('Creating your repository…')).toBeInTheDocument();
    });

    it('renders the overlay sub-message', () => {
        render(CreationOverlay, { props: { visible: true } });
        expect(
            screen.getByText("This may take up to a minute. Please don't close this page."),
        ).toBeInTheDocument();
    });

    // -------------------------------------------------------------------------
    // Accessibility
    // -------------------------------------------------------------------------

    it('dialog has aria-modal="true"', () => {
        render(CreationOverlay, { props: { visible: true } });
        expect(screen.getByRole('dialog')).toHaveAttribute('aria-modal', 'true');
    });

    it('spinner has role="status" and aria-label="Creating repository"', () => {
        render(CreationOverlay, { props: { visible: true } });
        expect(screen.getByRole('status', { name: 'Creating repository' })).toBeInTheDocument();
    });

    it('overlay content container has aria-live="polite"', () => {
        render(CreationOverlay, { props: { visible: true } });
        const content = screen
            .getByText('Creating your repository…')
            .closest('[aria-live]') as HTMLElement;
        expect(content).toHaveAttribute('aria-live', 'polite');
    });
});
