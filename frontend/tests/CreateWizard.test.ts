import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CreatePage from '../src/routes/create/+page.svelte';
import type { BrandConfig } from '../src/lib/types/brand';
import type { TemplateSummary } from '../src/lib/api/types';
import {
  ApiConflictError,
  ApiAuthError,
  ApiNetworkError,
  ApiServerError,
} from '../src/lib/api/errors';

// ---------------------------------------------------------------------------
// Mock the API client so no real HTTP requests are made
// ---------------------------------------------------------------------------

vi.mock('../src/lib/api/client', () => ({
  listTemplates: vi.fn(),
  getTemplateDetails: vi.fn(),
  validateRepositoryName: vi.fn(),
  listRepositoryTypes: vi.fn(),
  listTeams: vi.fn(),
  createRepository: vi.fn(),
}));

import { listTemplates, getTemplateDetails, listRepositoryTypes, listTeams } from '../src/lib/api/client';

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
    vi.mocked(listRepositoryTypes).mockResolvedValue([]);
    vi.mocked(listTeams).mockResolvedValue([]);
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

    await fireEvent.click(screen.getByRole('radio', { name: 'rust-library' }));
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
    await fireEvent.click(screen.getByRole('radio', { name: 'rust-library' }));
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
    await fireEvent.click(screen.getByRole('radio', { name: 'rust-library' }));
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
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
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
    await fireEvent.click(screen.getByRole('radio', { name: 'rust-library' }));
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

  // -------------------------------------------------------------------------
  // Step 3: Template variables (UX-ASSERT-015, UX-ASSERT-016, UX-ASSERT-017)
  // -------------------------------------------------------------------------

  it('shows "Template variables" heading on step 3', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'A Python service template', tags: [] },
      variables: [{ name: 'service_name', required: true }],
    });
    render(CreatePage, { props: makeProps() });

    // Advance to step 2
    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    const nextSettings = screen.getByRole('button', { name: /next: repository settings/i });
    await waitFor(() => expect(nextSettings).not.toBeDisabled());
    await fireEvent.click(nextSettings);

    // Verify step 2 shows "Next: Variables →" button
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
    expect(screen.getByRole('button', { name: /next: variables/i })).toBeInTheDocument();
  });

  it('shows VariableField inputs on step 3', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'Python service', tags: [] },
      variables: [
        { name: 'service_name', required: true },
        { name: 'owner_team', required: false, default_value: 'platform' },
      ],
    });
    render(CreatePage, { props: makeProps() });

    // Advance through step 1 and 2 to step 3
    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );

    // Click "Next: Variables →"
    await fireEvent.click(screen.getByRole('button', { name: /next: variables/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Template variables'),
    );

    // Both variable fields should be present
    expect(screen.getByRole('textbox', { name: /service name/i })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /owner team/i })).toBeInTheDocument();
  });

  it('pre-populates optional variable with default_value (UX-ASSERT-017)', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'Python service', tags: [] },
      variables: [
        { name: 'service_name', required: true },
        { name: 'owner_team', required: false, default_value: 'platform' },
      ],
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: variables/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Template variables'),
    );

    expect(screen.getByRole('textbox', { name: /owner team/i })).toHaveValue('platform');
  });

  it('disables Create button when a required variable is empty (UX-ASSERT-016)', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'Python service', tags: [] },
      variables: [{ name: 'service_name', required: true }],
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: variables/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Template variables'),
    );

    // Required field is empty → Create button disabled
    expect(screen.getByRole('button', { name: /create repository/i })).toBeDisabled();
  });

  it('enables Create button once all required variables are filled (UX-ASSERT-016)', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'Python service', tags: [] },
      variables: [{ name: 'service_name', required: true }],
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: variables/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Template variables'),
    );

    // Fill the required field
    const input = screen.getByRole('textbox', { name: /service name/i });
    await fireEvent.input(input, { target: { value: 'my-service' } });

    await waitFor(() =>
      expect(screen.getByRole('button', { name: /create repository/i })).not.toBeDisabled(),
    );
  });

  it('shows RepositorySummary on step 3', async () => {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'python-service',
      metadata: { description: 'Python service', tags: [] },
      variables: [{ name: 'service_name', required: true }],
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'python-service' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'python-service' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: variables/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Template variables'),
    );

    // RepositorySummary should be present with its aria-label
    expect(screen.getByRole('generic', { name: /repository summary/i })).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Creation overlay (CMP-011) and error routing (UX-ASSERT-018–021)
  // -------------------------------------------------------------------------

  /** Helper: advance to Step 2 with a no-variable template and mock createRepository. */
  async function advanceToStep2NoVars() {
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'rust-library',
      metadata: { description: 'A Rust library template', tags: [] },
      variables: [],
    });
    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'rust-library' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'rust-library' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
  }

  it('shows overlay while creation is in flight (UX-ASSERT-018)', async () => {
    // createRepository never resolves during this test
    vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
    vi.mocked(getTemplateDetails).mockResolvedValue({
      name: 'rust-library',
      metadata: { description: 'Template', tags: [] },
      variables: [],
    });
    const { createRepository } = await import('../src/lib/api/client');
    vi.mocked(createRepository).mockReturnValue(new Promise(() => {})); // never resolves

    render(CreatePage, { props: makeProps() });

    await waitFor(() => screen.getByRole('radio', { name: 'rust-library' }));
    await fireEvent.click(screen.getByRole('radio', { name: 'rust-library' }));
    await waitFor(() => expect(getTemplateDetails).toHaveBeenCalled());
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /next: repository settings/i })).not.toBeDisabled(),
    );
    await fireEvent.click(screen.getByRole('button', { name: /next: repository settings/i }));
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );

    // Click "Create Repository" — overlay should appear
    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));
    await waitFor(() =>
      expect(screen.getByRole('dialog', { name: /creating repository/i })).toBeInTheDocument(),
    );
  });

  it('returns to step 2 with error on name-taken race condition (UX-ASSERT-019)', async () => {
    const { createRepository } = await import('../src/lib/api/client');
    vi.mocked(createRepository).mockRejectedValue(
      new ApiConflictError(422, { code: 'NameTaken', message: 'Name already taken' }),
    );

    await advanceToStep2NoVars();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings'),
    );
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(screen.getByRole('alert').textContent).toMatch(/was created while you were filling/i);
  });

  it('returns to step 1 with error on template-not-found (UX-ASSERT-020)', async () => {
    const { createRepository } = await import('../src/lib/api/client');
    vi.mocked(createRepository).mockRejectedValue(
      new ApiConflictError(422, {
        code: 'TemplateNotFound',
        message: 'Template not found',
      }),
    );

    await advanceToStep2NoVars();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Choose a template'),
    );
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(screen.getByRole('alert').textContent).toMatch(/no longer available/i);
  });

  it('shows permission error inline on 403 (UX-ASSERT-021)', async () => {
    const { createRepository } = await import('../src/lib/api/client');
    vi.mocked(createRepository).mockRejectedValue(
      new ApiAuthError(403, { code: 'Forbidden', message: 'Forbidden' }),
    );

    await advanceToStep2NoVars();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(screen.getByRole('alert').textContent).toMatch(/don't have permission/i);
    // Still on step 2
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings');
  });

  it('shows network error inline without losing step (UX-ASSERT-021)', async () => {
    const { createRepository } = await import('../src/lib/api/client');
    vi.mocked(createRepository).mockRejectedValue(new ApiNetworkError(new Error('offline')));

    await advanceToStep2NoVars();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(screen.getByRole('alert').textContent).toMatch(/could not reach the server/i);
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings');
  });

  it('shows server error inline on 5xx (UX-ASSERT-021)', async () => {
    const { createRepository } = await import('../src/lib/api/client');
    vi.mocked(createRepository).mockRejectedValue(
      new ApiServerError(500, { code: 'InternalError', message: 'Internal server error' }),
    );

    await advanceToStep2NoVars();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(screen.getByRole('alert').textContent).toMatch(/GitHub is temporarily unavailable/i);
    expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Repository settings');
  });
});
