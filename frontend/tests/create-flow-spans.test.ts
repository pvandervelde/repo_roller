/**
 * Verifies that the /create wizard's two most important operations — loading
 * the template list on mount, and submitting the "Create Repository" form —
 * are wrapped in named spans via `src/lib/observability/spans.ts`'s
 * `withSpan()`, per the task's "OpenTelemetry JS SDK instrumentation ...
 * form submission (the /create flow), API call tracking" requirement.
 *
 * `src/routes/create/+page.svelte` is rendered unmodified (it is not this
 * Tester's to edit); these tests mock `spans.ts` and the API client, then
 * drive the real component through the DOM, asserting `withSpan` is called
 * with the correct `SPAN_NAMES` constant around the right operation.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import CreatePage from '../src/routes/create/+page.svelte';
import type { BrandConfig } from '../src/lib/types/brand';
import type { TemplateSummary } from '../src/lib/api/types';

vi.mock('../src/lib/api/client', () => ({
  listTemplates: vi.fn(),
  getTemplateDetails: vi.fn(),
  validateRepositoryName: vi.fn(),
  listRepositoryTypes: vi.fn(),
  listTeams: vi.fn(),
  createRepository: vi.fn(),
}));

vi.mock('../src/lib/observability/spans', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../src/lib/observability/spans')>();
  return {
    ...actual,
    withSpan: vi.fn(async (_name: string, fn: (span: unknown) => unknown) => fn({})),
  };
});

import {
  listTemplates,
  getTemplateDetails,
  listRepositoryTypes,
  listTeams,
  validateRepositoryName,
  createRepository,
} from '../src/lib/api/client';
import { withSpan, SPAN_NAMES } from '../src/lib/observability/spans';

const brandConfig: BrandConfig = {
  appName: 'RepoRoller',
  logoUrl: null,
  logoUrlDark: null,
  logoAlt: 'RepoRoller logo',
  primaryColor: '#0969da',
  primaryColorDark: null,
};

const mockTemplates: TemplateSummary[] = [
  { name: 'rust-library', description: 'A Rust library template', variables: [] },
];

function makeProps() {
  return {
    data: {
      brandConfig,
      session: { userLogin: 'jdoe', userAvatarUrl: null, backendToken: 'test-jwt' },
      userLogin: 'jdoe',
      organization: 'test-org',
    },
  };
}

beforeEach(() => {
  vi.mocked(withSpan).mockClear();
  vi.mocked(listTemplates).mockResolvedValue(mockTemplates);
  vi.mocked(getTemplateDetails).mockResolvedValue({
    name: 'rust-library',
    description: 'A Rust library template',
    variables: [],
  });
  vi.mocked(listRepositoryTypes).mockResolvedValue([]);
  vi.mocked(listTeams).mockResolvedValue([]);
  vi.mocked(validateRepositoryName).mockResolvedValue({ valid: true, available: true });
  vi.mocked(createRepository).mockResolvedValue({
    repository: {
      name: 'my-new-service',
      fullName: 'test-org/my-new-service',
      url: 'https://github.com/test-org/my-new-service',
      visibility: 'private',
    },
    appliedConfiguration: {},
    createdAt: '2026-01-01T00:00:00Z',
  });
});

describe('/create wizard — template loading is wrapped in a span', () => {
  it('calls withSpan with SPAN_NAMES.LOAD_TEMPLATES when loading templates on mount', async () => {
    render(CreatePage, { props: makeProps() });

    await waitFor(() => expect(listTemplates).toHaveBeenCalled());

    // Attributes (3rd arg) are optional per the spans.ts contract, so only
    // the span name and the wrapped function are asserted here.
    const [name, fn] = vi.mocked(withSpan).mock.calls[0] ?? [];
    expect(name).toBe(SPAN_NAMES.LOAD_TEMPLATES);
    expect(fn).toBeInstanceOf(Function);
  });
});

describe('/create wizard — repository creation submission is wrapped in a span', () => {
  async function advanceToStep2AndFillName() {
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

    const input = screen.getByRole('textbox', { name: /repository name/i });
    await fireEvent.input(input, { target: { value: 'my-new-service' } });
    await fireEvent.blur(input);
    await waitFor(() =>
      expect(screen.getByRole('button', { name: /create repository/i })).not.toBeDisabled(),
    );
  }

  it('calls withSpan with SPAN_NAMES.CREATE_REPOSITORY_SUBMIT when "Create Repository" is clicked', async () => {
    await advanceToStep2AndFillName();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() => expect(createRepository).toHaveBeenCalled());
    const submitCall = vi
      .mocked(withSpan)
      .mock.calls.find(([name]) => name === SPAN_NAMES.CREATE_REPOSITORY_SUBMIT);
    expect(submitCall).toBeDefined();
    expect(submitCall?.[1]).toBeInstanceOf(Function);
  });

  it('does not use the LOAD_TEMPLATES span name for the submit operation (stub-killing)', async () => {
    await advanceToStep2AndFillName();
    vi.mocked(withSpan).mockClear();

    await fireEvent.click(screen.getByRole('button', { name: /create repository/i }));

    await waitFor(() => expect(createRepository).toHaveBeenCalled());
    const callsWithLoadTemplatesName = vi
      .mocked(withSpan)
      .mock.calls.filter(([name]) => name === SPAN_NAMES.LOAD_TEMPLATES);
    expect(callsWithLoadTemplatesName).toHaveLength(0);
  });
});
