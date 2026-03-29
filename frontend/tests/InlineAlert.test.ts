import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import InlineAlert from '../src/lib/components/InlineAlert.svelte';

describe('InlineAlert', () => {
    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    it('renders the message text', () => {
        render(InlineAlert, { props: { variant: 'error', message: 'Something went wrong' } });
        expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    });

    it('renders an action button when the action prop is provided', () => {
        render(InlineAlert, {
            props: {
                variant: 'error',
                message: 'Failed',
                action: { label: 'Try again', onClick: vi.fn() },
            },
        });
        expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
    });

    it('does not render an action button when action is null', () => {
        render(InlineAlert, { props: { variant: 'error', message: 'Failed', action: null } });
        expect(screen.queryByRole('button')).toBeNull();
    });

    it('does not render an action button when action is omitted', () => {
        render(InlineAlert, { props: { variant: 'info', message: 'Note' } });
        expect(screen.queryByRole('button')).toBeNull();
    });

    // -------------------------------------------------------------------------
    // ARIA roles — UX-ASSERT-025
    // -------------------------------------------------------------------------

    it('uses role="alert" for error variant', () => {
        render(InlineAlert, { props: { variant: 'error', message: 'Error message' } });
        expect(screen.getByRole('alert')).toBeInTheDocument();
    });

    it('uses role="alert" for warning variant', () => {
        render(InlineAlert, { props: { variant: 'warning', message: 'Warning message' } });
        expect(screen.getByRole('alert')).toBeInTheDocument();
    });

    it('uses role="status" for info variant', () => {
        render(InlineAlert, { props: { variant: 'info', message: 'Info message' } });
        expect(screen.getByRole('status')).toBeInTheDocument();
    });

    it('uses role="status" for success variant', () => {
        render(InlineAlert, { props: { variant: 'success', message: 'Done!' } });
        expect(screen.getByRole('status')).toBeInTheDocument();
    });

    // -------------------------------------------------------------------------
    // Action callback
    // -------------------------------------------------------------------------

    it('calls action.onClick when the action button is clicked', async () => {
        const onClick = vi.fn();
        render(InlineAlert, {
            props: { variant: 'error', message: 'Failed', action: { label: 'Retry', onClick } },
        });
        await fireEvent.click(screen.getByRole('button', { name: 'Retry' }));
        expect(onClick).toHaveBeenCalledOnce();
    });

    // -------------------------------------------------------------------------
    // Variant classes (used by CSS for colour differentiation)
    // -------------------------------------------------------------------------

    it('applies a data-variant attribute matching the variant prop', () => {
        const { container } = render(InlineAlert, {
            props: { variant: 'success', message: 'Done' },
        });
        const alert = container.querySelector('[data-variant]');
        expect(alert?.getAttribute('data-variant')).toBe('success');
    });

    it('applies data-variant="error" for error variant', () => {
        const { container } = render(InlineAlert, { props: { variant: 'error', message: 'Oops' } });
        expect(container.querySelector('[data-variant="error"]')).toBeTruthy();
    });
});
