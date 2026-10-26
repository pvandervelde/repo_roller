import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import TemplateDetailsPanel from '../src/lib/components/TemplateDetailsPanel.svelte';
import type { GetTemplateDetailsResponse } from '../src/lib/api/types';

// ---------------------------------------------------------------------------
// RED-phase spec-gap notes (Tester → Coder):
//
// `TemplateDetailsPanel.svelte` is a genuinely new component (no existing
// sidebar/review pattern in the codebase per the Catalog Slice). The
// Interface Contract fixes the prop shape and the high-level content list
// but not exact markup. This file documents the structural assumptions the
// tests rely on; the Coder may pick different class names / landmark roles
// as long as the corresponding test(s) are updated to match (same latitude
// granted to the #286 RED phase for its documented spec gaps):
//
//   - Root element is a landmark reachable via
//     role="complementary" name="Template details" (an <aside aria-label=
//     "Template details">), present in all four states (empty / loading /
//     error / content) so parent layout code has one stable mount point.
//   - displayName renders as a level-2 heading (<h2>) — the page h1 belongs
//     to the wizard step, not the sidebar.
//   - Tags render in a <ul aria-label="Tags"> of <li> items — reusing
//     TemplateCard's aria-label convention for tags — but UNCAPPED (all
//     tags, not TemplateCard's 4-tag slice).
//   - Type badge, when rendered, reuses TemplateCard's exact class name
//     `.template-card__type-badge` and `data-policy` attribute (per Catalog
//     Slice instruction to reuse, not reinvent) and is omitted entirely
//     (no element at all) when `repository_type` is absent.
//   - Variables render in a <ul aria-label="Template variables"> of <li>
//     items. Each item shows: the variable name as visible text, a
//     "Required" or "Optional" indicator as visible text, the variable's
//     `description` if present, "Default: {value}" if `default` is present,
//     and "Pattern: {value}" if `pattern` is present — with each of the
//     optional lines omitted (not rendered as an empty line) when absent.
//   - Loading state: role="status" aria-live="polite" region containing the
//     exact text "Loading template details..." (per Interface Contract),
//     shown instead of any stale/previous content — even if `templateDetails`
//     is still non-null while a refetch is in flight.
//   - Error state: role="alert" containing the `error` message, shown
//     instead of variables/tags content.
//   - Empty state (no selection yet): a light role="status" hint — exact
//     copy is a free choice for the Coder; tests only assert that no
//     template content (heading/description/tags/variables) leaks through.
// ---------------------------------------------------------------------------

const baseTemplateDetails: GetTemplateDetailsResponse = {
  name: 'rust-library',
  displayName: 'Rust Library',
  description:
    'A comprehensive Rust library template with CI, linting, and release automation already wired up for you.',
  tags: ['rust', 'library', 'ci', 'linting', 'release-automation', 'cargo'],
  variables: [],
};

describe('TemplateDetailsPanel (new component)', () => {
  // -------------------------------------------------------------------------
  // Empty state — no template selected yet
  // -------------------------------------------------------------------------

  it('does not render any template content when templateDetails is null and not loading', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: null, loading: false } });
    expect(screen.queryByText(baseTemplateDetails.displayName)).toBeNull();
    expect(screen.queryByRole('heading', { level: 2 })).toBeNull();
  });

  it('does not show the loading indicator in the empty state', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: null, loading: false } });
    expect(screen.queryByText('Loading template details...')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Loading state
  // -------------------------------------------------------------------------

  it('shows a role="status" region with "Loading template details..." when loading=true', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: null, loading: true } });
    const status = screen.getByText('Loading template details...').closest('[role="status"]');
    expect(status).toBeTruthy();
  });

  it('marks the loading region aria-live="polite"', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: null, loading: true } });
    const status = screen.getByText('Loading template details...').closest('[aria-live]');
    expect(status).toHaveAttribute('aria-live', 'polite');
  });

  it('does not render stale template content while loading=true, even if templateDetails is still set', () => {
    // Guards against a lazy implementation that forgets to prioritise the
    // loading flag over already-present (now stale) details during a
    // template-switch refetch.
    render(TemplateDetailsPanel, {
      props: { templateDetails: baseTemplateDetails, loading: true },
    });
    expect(screen.queryByRole('heading', { level: 2, name: 'Rust Library' })).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Error state
  // -------------------------------------------------------------------------

  it('shows a role="alert" with the error message when error is set', () => {
    render(TemplateDetailsPanel, {
      props: { templateDetails: null, loading: false, error: 'Could not load template details.' },
    });
    expect(screen.getByRole('alert')).toHaveTextContent('Could not load template details.');
  });

  it('does not render variable/tag content alongside the error state', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: baseTemplateDetails,
        loading: false,
        error: 'Could not load template details.',
      },
    });
    expect(screen.queryByRole('heading', { level: 2 })).toBeNull();
  });

  // ---------------------------------------------------------------------------
  // AUDIT: branch-priority — loading vs. error, both truthy at once.
  //
  // Every other state-priority pair in this file is exercised with both
  // conditions simultaneously truthy (loading+content above; error+content
  // here), which is what actually proves the `{#if}/{:else if}` ordering
  // rather than just the individual branches. The loading/error pair was
  // the one combination never exercised that way — a mutant that swapped
  // the `{#if loading}` and `{:else if error}` branches (checking error
  // first) would still pass every existing test, since no existing test
  // sets both `loading` and `error` truthy at the same time.
  // ---------------------------------------------------------------------------
  it('prioritises the loading state over the error state when both are truthy (kills a loading/error branch-swap mutant)', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: null,
        loading: true,
        error: 'Could not load template details.',
      },
    });
    expect(screen.getByText('Loading template details...')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Content state — heading / description
  // -------------------------------------------------------------------------

  it('renders displayName as a level-2 heading', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: baseTemplateDetails, loading: false } });
    expect(screen.getByRole('heading', { level: 2 })).toHaveTextContent('Rust Library');
  });

  it('renders the full, untruncated description text', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: baseTemplateDetails, loading: false } });
    expect(screen.getByText(baseTemplateDetails.description)).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Tags — uncapped (unlike TemplateCard's 4-tag slice)
  // -------------------------------------------------------------------------

  it('renders a tags list labelled "Tags"', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: baseTemplateDetails, loading: false } });
    expect(screen.getByRole('list', { name: 'Tags' })).toBeInTheDocument();
  });

  it('renders every tag, not just the first 4 (regression against TemplateCard\'s cap)', () => {
    render(TemplateDetailsPanel, { props: { templateDetails: baseTemplateDetails, loading: false } });
    for (const tag of baseTemplateDetails.tags) {
      expect(screen.getByText(tag)).toBeInTheDocument();
    }
  });

  it('renders no tags list when the tags array is empty', () => {
    render(TemplateDetailsPanel, {
      props: { templateDetails: { ...baseTemplateDetails, tags: [] }, loading: false },
    });
    expect(screen.queryByRole('list', { name: 'Tags' })).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Repository-type badge — omitted when absent, reuses TemplateCard's class
  // -------------------------------------------------------------------------

  it('omits the type badge entirely when repository_type is absent', () => {
    const { container } = render(TemplateDetailsPanel, {
      props: { templateDetails: baseTemplateDetails, loading: false },
    });
    expect(container.querySelector('.template-card__type-badge')).toBeNull();
  });

  it('renders the type badge with the correct data-policy attribute when repository_type is present', () => {
    const { container } = render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          repository_type: { policy: 'fixed', type_name: 'service' },
        },
        loading: false,
      },
    });
    const badge = container.querySelector('.template-card__type-badge');
    expect(badge).toHaveAttribute('data-policy', 'fixed');
  });

  it('renders the type badge text equal to type_name', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          repository_type: { policy: 'preferable', type_name: 'microservice' },
        },
        loading: false,
      },
    });
    expect(screen.getByText('microservice')).toBeInTheDocument();
  });

  it('renders a different data-policy value when the policy changes (kills a hardcoded "fixed" stub)', () => {
    const { container } = render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          repository_type: { policy: 'optional', type_name: 'library' },
        },
        loading: false,
      },
    });
    const badge = container.querySelector('.template-card__type-badge');
    expect(badge).toHaveAttribute('data-policy', 'optional');
  });

  // -------------------------------------------------------------------------
  // Variables — read-only preview
  // -------------------------------------------------------------------------

  it('renders a variables list labelled "Template variables" when variables are present', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [{ name: 'service_name', required: true }],
        },
        loading: false,
      },
    });
    expect(screen.getByRole('list', { name: 'Template variables' })).toBeInTheDocument();
  });

  it('does not render a variables list when the template has no variables', () => {
    render(TemplateDetailsPanel, {
      props: { templateDetails: { ...baseTemplateDetails, variables: [] }, loading: false },
    });
    expect(screen.queryByRole('list', { name: 'Template variables' })).toBeNull();
  });

  it('renders each variable\'s name', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [
            { name: 'service_name', required: true },
            { name: 'owner_team', required: false, default: 'platform' },
          ],
        },
        loading: false,
      },
    });
    expect(screen.getByText('service_name')).toBeInTheDocument();
    expect(screen.getByText('owner_team')).toBeInTheDocument();
  });

  it('shows a "Required" indicator for required variables', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: { ...baseTemplateDetails, variables: [{ name: 'service_name', required: true }] },
        loading: false,
      },
    });
    expect(screen.getByText('Required')).toBeInTheDocument();
  });

  it('shows an "Optional" indicator for non-required variables', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [{ name: 'owner_team', required: false }],
        },
        loading: false,
      },
    });
    expect(screen.getByText('Optional')).toBeInTheDocument();
  });

  it('shows the variable description when present', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [
            { name: 'service_name', required: true, description: 'The name of the microservice' },
          ],
        },
        loading: false,
      },
    });
    expect(screen.getByText('The name of the microservice')).toBeInTheDocument();
  });

  it('shows "Default: {value}" when a default is present', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [{ name: 'owner_team', required: false, default: 'platform' }],
        },
        loading: false,
      },
    });
    expect(screen.getByText(/Default:\s*platform/)).toBeInTheDocument();
  });

  it('does not render a "Default:" line when no default is present', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: { ...baseTemplateDetails, variables: [{ name: 'service_name', required: true }] },
        loading: false,
      },
    });
    expect(screen.queryByText(/Default:/)).toBeNull();
  });

  it('shows "Pattern: {value}" when a pattern is present', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [{ name: 'service_name', required: true, pattern: '^[a-z0-9-]+$' }],
        },
        loading: false,
      },
    });
    expect(screen.getByText(/Pattern:\s*\^\[a-z0-9-\]\+\$/)).toBeInTheDocument();
  });

  it('does not render a "Pattern:" line when no pattern is present', () => {
    render(TemplateDetailsPanel, {
      props: {
        templateDetails: { ...baseTemplateDetails, variables: [{ name: 'service_name', required: true }] },
        loading: false,
      },
    });
    expect(screen.queryByText(/Pattern:/)).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Security — never render via {@html} (matches the #286 convention)
  // -------------------------------------------------------------------------

  it('renders a description containing markup-like text as literal text, not parsed HTML', () => {
    const malicious = '<img src=x onerror="window.__pwned=true">';
    const { container } = render(TemplateDetailsPanel, {
      props: { templateDetails: { ...baseTemplateDetails, description: malicious }, loading: false },
    });
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toContain(malicious);
  });

  it('renders a tag containing markup-like text as literal text, not parsed HTML', () => {
    const malicious = '<script>window.__pwned=true</script>';
    const { container } = render(TemplateDetailsPanel, {
      props: { templateDetails: { ...baseTemplateDetails, tags: [malicious] }, loading: false },
    });
    expect(container.querySelector('script')).toBeNull();
    expect(container.textContent).toContain(malicious);
  });

  it('renders a variable name containing markup-like text as literal text, not parsed HTML', () => {
    const malicious = '<img src=x onerror="window.__pwned=true">';
    const { container } = render(TemplateDetailsPanel, {
      props: {
        templateDetails: {
          ...baseTemplateDetails,
          variables: [{ name: malicious, required: true }],
        },
        loading: false,
      },
    });
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toContain(malicious);
  });
});
