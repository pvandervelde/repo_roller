import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CreatePage from '../src/routes/create/+page.svelte';
import type { BrandConfig } from '../src/lib/types/brand';
import type { TemplateSummary } from '../src/lib/api/types';

// ---------------------------------------------------------------------------
// Mock the API client so no real HTTP requests are made
// ---------------------------------------------------------------------------

vi.mock('../src/lib/api/client', () => ({
  listTemplates: vi.fn(),
  getTemplateDetails: vi.fn(),
}));

import { listTemplates, getTemplateDetails } from '../src/lib/api/client';

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const brandConfig: BrandConfig = {
  appName: 'RepoRoller',
  logoUrl: null,
  logoUrlDark: null,
  logoAlt: 'RepoRoller logo',
  primaryColor: '#0969da',
  primaryColorDark: null,
};

const mockTemplates: TemplateSummary[] = [
  { name: 'rust-library', description: 'A Rust library template', tags: ['rust'] },
  { name: 'python-service', description: 'A Python service template', tags: ['python'] },
];

function makeProps() {
  return {
    data: {
      brandConfig,
      session: { userLogin: 'jdoe', userAvatarUrl: null },
      userLogin: 'jdoe',
      organization: 'test-org',
    },
  };
}

describe('Create wizard (SCR-004)', () => {
  beforeEach(() => {
    vi.mocked(listTemplates).mockResolvedValue([]);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'rust-library',
      metadata: { description: 'A Rust library template', tags: ['rust'] },
      variables: [],
    });
  });

  // -------------------------------------------------------------------------
  // Page title
  // -------------------------------------------------------------------------

  it('sets the correct page title', () => {
    render(CreatePage, { props: makeProps() });
    expect(document.title).toBe('Create repository — RepoRoller');
  });

  // -------------------------------------------------------------------------
  // Step 1: initial state
  // -------------------------------------------------------------------------

  it('renders the "Choose a template" heading on initial load', () => {
    render(CreatePage, { props: makeProps() });
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Choose a template');
  });

  it('renders the StepProgress with step 1 aria-label', () => {
    render(CreatePage, { props: makeProps() });
    expect(screen.getByRole('group', { name: /step 1 of/i })).toBeInTheDocument();
  });

  it('loads templates from the API on mount', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    render(CreatePage, { props: makeProps() });
    await waitFor(() => expect(listTemplates).toHaveBeenCalledWith('test-org'));
  });

  it('"Next" button is disabled when no template is selected', () => {
    render(CreatePage, { props: makeProps() });
    expect(screen.getByRole('button', { name: /next: repository settings/i })).toBeDisabled();
  });

  it('does not show a "← Back" button on step 1', () => {
    render(CreatePage, { props: makeProps() });
    expect(screen.queryByRole('button', { name: /← back/i })).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Step navigation
  // -------------------------------------------------------------------------

  it('shows "Repository settings" heading after advancing to step 2', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    render(CreatePage, { props: makeProps() });

    // Wait for templates to load, then select one to trigger detail fetch
    await waitFor(() => expect(listTemplates).toHaveBeenCalled());
    await waitFor(() => screen.getByRole('radio', { name: 'rust-library' }));

    await fireEvent.change(screen.getByRole('radio', { name: 'rust-library' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());

    const next = screen.getByRole('button', { name: /next: repository settings/i });
    await waitFor(() => expect(next).not.toBeDisabled());
    await fireEvent.click(next);

    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
  });

  it('returns to step 1 when "← Back" is clicked from step 2', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    render(CreatePage, { props: makeProps() });

    // Advance to step 2
    await waitFor(() => screen.getByRole('radio', { name: 'rust-library' }));
    await fireEvent.change(screen.getByRole('radio', { name: 'rust-library' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    const next = screen.getByRole('button', { name: /next: repository settings/i });
    await waitFor(() => expect(next).not.toBeDisabled());
    await fireEvent.click(next);
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );

    // Back to step 1
    await fireEvent.click(screen.getByRole('button', { name: /← back/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Choose a template'),
    );
  });

  it('shows StepProgress with 2 steps when template has no variables', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'rust-library',
      metadata: { description: 'A Rust library template', tags: [] },
      variables: [], // no variables
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'rust-library' }));
    await fireEvent.change(screen.getByRole('radio', { name: 'rust-library' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());

    // 2-step: aria-label should say "of 2"
    expect(screen.getByRole('group', { name: /step 1 of 2/i })).toBeInTheDocument();
  });

  it('shows StepProgress with 3 steps when template has variables', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'A Python service template', tags: [] },
      variables: [{ name: 'service_name', required: true }],
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.change(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());

    // 3-step: aria-label should say "of 3"
    expect(screen.getByRole('group', { name: /step 1 of 3/i })).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Focus management — UX-ASSERT-024
  // -------------------------------------------------------------------------

  it('moves focus to the step 2 heading when advancing', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'rust-library' }));
    await fireEvent.change(screen.getByRole('radio', { name: 'rust-library' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    const next = screen.getByRole('button', { name: /next: repository settings/i });
    await waitFor(() => expect(next).not.toBeDisabled());
    await fireEvent.click(next);

    await waitFor(() => {
      const h1 = screen.getByRole('heading', { level: 1 });
      expect(h1).toHaveTextContent('Repository settings');
      expect(document.activeElement).toBe(h1);
    });
  });
});
