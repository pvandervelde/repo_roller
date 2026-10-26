import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import TemplateGrid from '../src/lib/components/TemplateGrid.svelte';
import type { TemplateSummary } from '../src/lib/api/types';

// #286 issue 2/3/4: `TemplateSummary` gains `displayName` (independent of the
// slug `name`) and a full `tags: string[]` array (replacing the old
// single-value `category`). Fixtures below are deliberately crafted so that
// name/description/tags/displayName matches don't accidentally overlap,
// letting each filtering test isolate exactly one match reason.
const templates: TemplateSummary[] = [
  {
    name: 'typescript-service',
    displayName: 'TypeScript Service',
    description: 'A fast backend microservice template',
    tags: ['typescript', 'node'],
    variables: [],
  },
  {
    name: 'python-library',
    displayName: 'Python Library',
    description: 'A reusable Python library',
    tags: ['python', 'library'],
    variables: [],
  },
  {
    name: 'frontend-app',
    displayName: 'Frontend Application',
    description: 'SvelteKit frontend application',
    tags: ['frontend', 'sveltekit'],
    variables: [],
  },
  {
    name: 'rust-microservice',
    displayName: 'Rust Microservice',
    description: 'Production-ready backend service',
    // 'grpc' is the *second* tag -- used to prove the full array is
    // searched, not just the first element (the old `category` bug).
    tags: ['rust', 'grpc', 'observability'],
    variables: [],
  },
  {
    name: 'proj-42',
    // Deliberately shares no substring with the slug, so matching on this
    // value alone proves displayName (not name) is being searched.
    displayName: 'Quantum Launcher',
    description: 'Internal tooling scaffold',
    tags: [],
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

  // ---------------------------------------------------------------------
  // #286 issue 4: search by tags (full array, not just the first tag) and
  // by displayName.
  // ---------------------------------------------------------------------

  it('matches a tag that is not the first element of the tags array (regression against first-tag-only search)', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    // 'grpc' is rust-microservice's *second* tag. A stub that only searches
    // `tags[0]` (mirroring the old single-`category` bug) would never match.
    await fireEvent.input(search, { target: { value: 'grpc' } });
    expect(screen.getByRole('radio', { name: 'rust-microservice' })).toBeInTheDocument();
    expect(screen.queryByRole('radio', { name: 'typescript-service' })).toBeNull();
    expect(screen.queryByRole('radio', { name: 'python-library' })).toBeNull();
    expect(screen.queryByRole('radio', { name: 'frontend-app' })).toBeNull();
  });

  it('matches tags case-insensitively', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    await fireEvent.input(search, { target: { value: 'GRPC' } });
    expect(screen.getByRole('radio', { name: 'rust-microservice' })).toBeInTheDocument();
  });

  it('matches displayName when the query matches the human-readable name but not the slug', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    // 'quantum' appears only in proj-42's displayName ('Quantum Launcher'),
    // never in its slug, description, or tags.
    await fireEvent.input(search, { target: { value: 'quantum' } });
    expect(screen.getByRole('radio', { name: 'proj-42' })).toBeInTheDocument();
    expect(screen.queryByRole('radio', { name: 'typescript-service' })).toBeNull();
    expect(screen.queryByRole('radio', { name: 'rust-microservice' })).toBeNull();
  });

  it('still matches by the slug name even when it differs from displayName', async () => {
    render(TemplateGrid, { props: { templates } });
    const search = screen.getByRole('searchbox');
    await fireEvent.input(search, { target: { value: 'proj-42' } });
    expect(screen.getByRole('radio', { name: 'proj-42' })).toBeInTheDocument();
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

  // -------------------------------------------------------------------------
  // #286 issue 2/3: full displayName/tags passthrough to TemplateCard.
  //
  // The grid currently wraps a single `category` into a one-element tags
  // array (`template.category ? [template.category] : []`) and never passes
  // `displayName` at all. These tests exercise the *full* multi-tag array
  // and the human-readable heading through the real child component, so a
  // stub that keeps wrapping a single value (or keeps using `name` for the
  // heading) fails even though each individual prop "looks" wired.
  // -------------------------------------------------------------------------

  it('passes the full tags array through to TemplateCard, not just a single wrapped value', () => {
    render(TemplateGrid, { props: { templates } });
    // rust-microservice has three tags; all three must be visible, not just
    // one derived the way the old `category` field was.
    expect(screen.getByText('rust')).toBeInTheDocument();
    expect(screen.getByText('grpc')).toBeInTheDocument();
    expect(screen.getByText('observability')).toBeInTheDocument();
  });

  it("passes each template's displayName through to TemplateCard's heading, not the slug", () => {
    render(TemplateGrid, { props: { templates } });
    // proj-42's displayName ('Quantum Launcher') shares no text with its
    // slug, so finding this heading proves displayName is actually wired
    // through the grid to the card, not silently defaulted to `name`.
    expect(screen.getByRole('heading', { level: 3, name: 'Quantum Launcher' })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { level: 3, name: 'proj-42' })).toBeNull();
  });

  it('renders no tags for a template whose tags array is empty', () => {
    render(TemplateGrid, { props: { templates } });
    const quantumRadio = screen.getByRole('radio', { name: 'proj-42' });
    const card = quantumRadio.closest('label');
    expect(card?.querySelector('.template-card__tags')).toBeNull();
  });
});
