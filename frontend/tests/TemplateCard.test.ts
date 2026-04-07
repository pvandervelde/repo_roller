import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import TemplateCard from '../src/lib/components/TemplateCard.svelte';
import type { RepositoryTypeBadge } from '../src/lib/components/TemplateCard.svelte';

const baseProps = {
    name: 'my-service-template',
    description: 'A template for microservices',
    selected: false,
};

describe('TemplateCard (CMP-004)', () => {
    // -------------------------------------------------------------------------
    // Rendering
    // -------------------------------------------------------------------------

    it('renders the template name', () => {
        render(TemplateCard, { props: baseProps });
        expect(screen.getByText('my-service-template')).toBeInTheDocument();
    });

    it('renders the template description', () => {
        render(TemplateCard, { props: baseProps });
        expect(screen.getByText('A template for microservices')).toBeInTheDocument();
    });

    it('renders up to 4 tags', () => {
        render(TemplateCard, {
            props: { ...baseProps, tags: ['typescript', 'node', 'api', 'rest'] },
        });
        expect(screen.getByText('typescript')).toBeInTheDocument();
        expect(screen.getByText('rest')).toBeInTheDocument();
    });

    it('does not render more than 4 tags when 5 are provided', () => {
        render(TemplateCard, {
            props: { ...baseProps, tags: ['a', 'b', 'c', 'd', 'fifth'] },
        });
        expect(screen.queryByText('fifth')).toBeNull();
    });

    it('renders the repository type badge when provided', () => {
        const badge: RepositoryTypeBadge = { typeName: 'service', policy: 'fixed' };
        render(TemplateCard, { props: { ...baseProps, repositoryTypeBadge: badge } });
        expect(screen.getByText('service')).toBeInTheDocument();
    });

    it('does not render a repository type badge when null', () => {
        const { container } = render(TemplateCard, {
            props: { ...baseProps, repositoryTypeBadge: null },
        });
        expect(container.querySelector('.template-card__type-badge')).toBeNull();
    });

    // -------------------------------------------------------------------------
    // Radio input semantics
    // -------------------------------------------------------------------------

    it('renders a radio input with name="template-selection"', () => {
        render(TemplateCard, { props: baseProps });
        expect(screen.getByRole('radio')).toHaveAttribute('name', 'template-selection');
    });

    it('radio input value equals the template name', () => {
        render(TemplateCard, { props: baseProps });
        expect(screen.getByRole('radio')).toHaveAttribute('value', 'my-service-template');
    });

    it('radio input is checked when selected=true', () => {
        render(TemplateCard, { props: { ...baseProps, selected: true } });
        expect(screen.getByRole('radio')).toBeChecked();
    });

    it('radio input is not checked when selected=false', () => {
        render(TemplateCard, { props: { ...baseProps, selected: false } });
        expect(screen.getByRole('radio')).not.toBeChecked();
    });

    it('radio input is wrapped in a label element', () => {
        render(TemplateCard, { props: baseProps });
        const radio = screen.getByRole('radio');
        expect(radio.closest('label')).toBeTruthy();
    });

    // -------------------------------------------------------------------------
    // Selected state
    // -------------------------------------------------------------------------

    it('shows a checkmark icon when selected=true', () => {
        const { container } = render(TemplateCard, { props: { ...baseProps, selected: true } });
        expect(container.querySelector('.template-card__checkmark')).toBeTruthy();
    });

    it('does not show a checkmark icon when selected=false', () => {
        const { container } = render(TemplateCard, { props: { ...baseProps, selected: false } });
        expect(container.querySelector('.template-card__checkmark')).toBeNull();
    });

    // -------------------------------------------------------------------------
    // Loading state
    // -------------------------------------------------------------------------

    it('does not render the template name when loading=true', () => {
        render(TemplateCard, { props: { ...baseProps, loading: true } });
        expect(screen.queryByText('my-service-template')).toBeNull();
    });

    it('renders a loading skeleton element when loading=true', () => {
        const { container } = render(TemplateCard, { props: { ...baseProps, loading: true } });
        expect(container.querySelector('.template-card--loading')).toBeTruthy();
    });

    // -------------------------------------------------------------------------
    // Interaction
    // -------------------------------------------------------------------------

    it('fires the onselect callback when the radio input is clicked (unselected state)', async () => {
        const onselect = vi.fn();
        render(TemplateCard, { props: { ...baseProps, selected: false, onselect } });
        await fireEvent.click(screen.getByRole('radio'));
        expect(onselect).toHaveBeenCalledOnce();
    });

    it('fires the ondeselect callback when the radio is clicked while already selected', async () => {
        const ondeselect = vi.fn();
        render(TemplateCard, { props: { ...baseProps, selected: true, ondeselect } });
        await fireEvent.click(screen.getByRole('radio'));
        expect(ondeselect).toHaveBeenCalledOnce();
    });

    it('does not fire onselect when the radio is clicked while already selected', async () => {
        const onselect = vi.fn();
        render(TemplateCard, { props: { ...baseProps, selected: true, onselect } });
        await fireEvent.click(screen.getByRole('radio'));
        expect(onselect).not.toHaveBeenCalled();
    });
});
