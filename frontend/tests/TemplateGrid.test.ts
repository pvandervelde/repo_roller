import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import TemplateGrid from '../src/lib/components/TemplateGrid.svelte';
import type { TemplateSummary } from '../src/lib/api/types';

const templates: TemplateSummary[] = [
  {
    name: 'typescript-service',
    description: 'A Node.js TypeScript microservice',
    category: 'typescript',
    variables: [],
  },
  {
    name: 'python-library',
    description: 'A reusable Python library',
    category: 'python',
    variables: [],
  },
  {
    name: 'frontend-app',
    description: 'SvelteKit frontend application',
    category: 'frontend',
    variables: [],
  },
];

describe('TemplateGrid (CMP-005)', () => {
  // -------------------------------------------------------------------------
  // Search bar
  // -------------------------------------------------------------------------

  it('renders a search input labeled "Search templates"', () => {
    render(TemplateGrid, { props: { templates } });
    expect(screen.getByRole('searchbox', { name: 'Search templates' })).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Radiogroup
  // -------------------------------------------------------------------------

  it('wraps template cards in a radiogroup labelled "Available templates"', () => {
    render(TemplateGrid, { props: { templates } });
    expect(screen.getByRole('radiogroup', { name: 'Available templates' })).toBeInTheDocument();
  });

  it('renders all templates when no search query is entered', () => {
    render(TemplateGrid, { props: { templates } });
    expect(screen.getByRole('radio', { name: 'typescript-service' })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: 'python-library' })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: 'frontend-app' })).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Filtering
  // -------------------------------------------------------------------------

  it('filters templates by name when a search query is entered', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    await fireEvent.input(search, { target: { value: 'python' } });
    expect(screen.getByRole('radio', { name: 'python-library' })).toBeInTheDocument();
    expect(screen.queryByRole('radio', { name: 'typescript-service' })).toBeNull();
  });

  it('filters templates by description when the query matches description text', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    await fireEvent.input(search, { target: { value: 'SvelteKit' } });
    expect(screen.getByRole('radio', { name: 'frontend-app' })).toBeInTheDocument();
    expect(screen.queryByRole('radio', { name: 'typescript-service' })).toBeNull();
  });

  it('filters templates by tag when the query matches a tag', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    await fireEvent.input(search, { target: { value: 'node' } });
    expect(screen.getByRole('radio', { name: 'typescript-service' })).toBeInTheDocument();
    expect(screen.queryByRole('radio', { name: 'python-library' })).toBeNull();
  });

  it('shows a "No templates match" message with role="status" when search finds nothing', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    await fireEvent.input(search, { target: { value: 'zzznomatch' } });
    const status = screen.getByRole('status');
    expect(status).toHaveTextContent("No templates match 'zzznomatch'");
  });

  // -------------------------------------------------------------------------
  // Loading state
  // -------------------------------------------------------------------------

  it('does not render selectable radio inputs when loading=true', () => {
    render(TemplateGrid, { props: { templates, loading: true } });
    expect(screen.queryByRole('radio')).toBeNull();
  });

  it('renders loading skeleton cards when loading=true', () => {
    const { container } = render(TemplateGrid, { props: { templates, loading: true } });
    expect(container.querySelector('.template-card--loading')).toBeTruthy();
  });

  // -------------------------------------------------------------------------
  // Error state — UX-ASSERT-008
  // -------------------------------------------------------------------------

  it('shows the error message when error prop is set', () => {
    render(TemplateGrid, {
      props: { templates, error: 'Could not load templates.' },
    });
    expect(screen.getByRole('alert')).toHaveTextContent('Could not load templates.');
  });

  it('does not render template cards when an error is present', () => {
    render(TemplateGrid, {
      props: { templates, error: 'Could not load templates.' },
    });
    expect(screen.queryByRole('radio')).toBeNull();
  });

  it('shows a "Try again" button in the error state', () => {
    render(TemplateGrid, {
      props: { templates, error: 'Could not load templates.' },
    });
    expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
  });

  it('fires onretry when "Try again" is clicked', async () => {
    const onretry = vi.fn();
    render(TemplateGrid, {
      props: { templates, error: 'Could not load templates.', onretry },
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(onretry).toHaveBeenCalledOnce();
  });

  // -------------------------------------------------------------------------
  // Empty state
  // -------------------------------------------------------------------------

  it('shows an empty state message when templates list is empty', () => {
    render(TemplateGrid, { props: { templates: [] } });
    expect(screen.getByRole('status')).toHaveTextContent(
      'No templates are configured for this organization.',
    );
  });

  // -------------------------------------------------------------------------
  // Selection — UX-ASSERT-005, UX-ASSERT-007
  // -------------------------------------------------------------------------

  it('fires ontemplateSelect with the template name when a radio is clicked', async () => {
    const ontemplateSelect = vi.fn();
    render(TemplateGrid, { props: { templates, ontemplateSelect } });
    await fireEvent.click(screen.getByRole('radio', { name: 'python-library' }));
    expect(ontemplateSelect).toHaveBeenCalledWith('python-library');
  });

  it('marks the card matching selectedTemplateName as selected', () => {
    render(TemplateGrid, {
      props: { templates, selectedTemplateName: 'frontend-app' },
    });
    expect(screen.getByRole('radio', { name: 'frontend-app' })).toBeChecked();
    expect(screen.getByRole('radio', { name: 'typescript-service' })).not.toBeChecked();
  });
});
