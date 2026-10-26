import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import RepositorySummary from '../src/lib/components/RepositorySummary.svelte';

const baseProps = {
  organization: 'acme',
  repositoryName: 'my-repo',
  templateName: 'rust-library',
  visibility: 'private' as const,
};

describe('RepositorySummary (CMP-009)', () => {
  // -------------------------------------------------------------------------
  // Repository identity
  // -------------------------------------------------------------------------

  it('shows the prefix "You are about to create:"', () => {
    render(RepositorySummary, { props: baseProps });
    expect(screen.getByText('You are about to create:')).toBeInTheDocument();
  });

  it('renders the organization name', () => {
    render(RepositorySummary, { props: baseProps });
    expect(screen.getByText('acme')).toBeInTheDocument();
  });

  it('renders the repository name', () => {
    render(RepositorySummary, { props: baseProps });
    expect(screen.getByText('my-repo')).toBeInTheDocument();
  });

  it('shows "—" placeholder when repositoryName is empty', () => {
    render(RepositorySummary, { props: { ...baseProps, repositoryName: '' } });
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Configuration chips
  // -------------------------------------------------------------------------

  it('shows a template chip', () => {
    render(RepositorySummary, { props: baseProps });
    expect(screen.getByText('Template')).toBeInTheDocument();
    expect(screen.getByText('rust-library')).toBeInTheDocument();
  });

  it('shows a visibility chip', () => {
    render(RepositorySummary, { props: baseProps });
    expect(screen.getByText('Visibility')).toBeInTheDocument();
    expect(screen.getByText('private')).toBeInTheDocument();
  });

  it('shows a type chip when typeName is provided', () => {
    render(RepositorySummary, { props: { ...baseProps, typeName: 'library' } });
    expect(screen.getByText('Type')).toBeInTheDocument();
    expect(screen.getByText('library')).toBeInTheDocument();
  });

  it('does not show a type chip when typeName is null', () => {
    render(RepositorySummary, { props: { ...baseProps, typeName: null } });
    expect(screen.queryByText('Type')).toBeNull();
  });

  it('shows a team chip when teamName is provided', () => {
    render(RepositorySummary, { props: { ...baseProps, teamName: 'platform-eng' } });
    expect(screen.getByText('Team')).toBeInTheDocument();
    expect(screen.getByText('platform-eng')).toBeInTheDocument();
  });

  it('does not show a team chip when teamName is null', () => {
    render(RepositorySummary, { props: { ...baseProps, teamName: null } });
    expect(screen.queryByText('Team')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Visibility variants
  // -------------------------------------------------------------------------

  it('renders "public" visibility correctly', () => {
    render(RepositorySummary, { props: { ...baseProps, visibility: 'public' as const } });
    expect(screen.getByText('public')).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // RED-phase spec-gap notes (Tester → Coder):
  //
  // The Interface Contract adds three new optional props (`description`,
  // `tags`, `variables`) for the new Review step, but does not fix exact
  // markup. These tests assume:
  //   - `description`, when provided, renders as plain text somewhere in the
  //     component (no specific element required beyond that).
  //   - `tags`, when provided, render in a <ul aria-label="Template tags">
  //     of <li> items, UNCAPPED (the Review step is a final summary, not a
  //     space-constrained card, so unlike TemplateCard's 4-tag slice every
  //     tag is shown).
  //   - `variables`, when provided, render in a
  //     <ul aria-label="Configured variables"> of <li> items, each showing
  //     both the variable's `name` and its `value` as visible text.
  // The Coder may choose different class/aria-label strings as long as the
  // corresponding assertions below are updated to match (same latitude
  // granted to the #286 RED phase for its documented spec gaps).
  // -------------------------------------------------------------------------

  describe('new optional props — description/tags/variables (Review step)', () => {
    it('renders the description text when provided', () => {
      render(RepositorySummary, {
        props: { ...baseProps, description: 'A template for Rust libraries.' },
      });
      expect(screen.getByText('A template for Rust libraries.')).toBeInTheDocument();
    });

    it('does not render a description region when the prop is omitted', () => {
      const { container } = render(RepositorySummary, { props: baseProps });
      expect(container.querySelector('.repo-summary__description')).toBeNull();
    });

    it('renders a tags list labelled "Template tags" when tags are provided', () => {
      render(RepositorySummary, { props: { ...baseProps, tags: ['rust', 'library'] } });
      expect(screen.getByRole('list', { name: 'Template tags' })).toBeInTheDocument();
    });

    it('renders every tag, not capped at 4, when 5 tags are provided (unlike TemplateCard)', () => {
      const tags = ['one', 'two', 'three', 'four', 'fifth-tag'];
      render(RepositorySummary, { props: { ...baseProps, tags } });
      for (const tag of tags) {
        expect(screen.getByText(tag)).toBeInTheDocument();
      }
    });

    it('does not render a tags list when the prop is omitted', () => {
      render(RepositorySummary, { props: baseProps });
      expect(screen.queryByRole('list', { name: 'Template tags' })).toBeNull();
    });

    it('does not render a tags list when an empty array is provided', () => {
      render(RepositorySummary, { props: { ...baseProps, tags: [] } });
      expect(screen.queryByRole('list', { name: 'Template tags' })).toBeNull();
    });

    it('renders a configured-variables list labelled "Configured variables" when variables are provided', () => {
      render(RepositorySummary, {
        props: { ...baseProps, variables: [{ name: 'service_name', value: 'my-service' }] },
      });
      expect(screen.getByRole('list', { name: 'Configured variables' })).toBeInTheDocument();
    });

    it('renders both the name and the chosen value for each configured variable', () => {
      render(RepositorySummary, {
        props: {
          ...baseProps,
          variables: [
            { name: 'service_name', value: 'my-service' },
            { name: 'owner_team', value: 'platform' },
          ],
        },
      });
      expect(screen.getByText('service_name')).toBeInTheDocument();
      expect(screen.getByText('my-service')).toBeInTheDocument();
      expect(screen.getByText('owner_team')).toBeInTheDocument();
      expect(screen.getByText('platform')).toBeInTheDocument();
    });

    it('does not render a configured-variables list when the prop is omitted', () => {
      render(RepositorySummary, { props: baseProps });
      expect(screen.queryByRole('list', { name: 'Configured variables' })).toBeNull();
    });

    it('does not render a configured-variables list when an empty array is provided', () => {
      render(RepositorySummary, { props: { ...baseProps, variables: [] } });
      expect(screen.queryByRole('list', { name: 'Configured variables' })).toBeNull();
    });

    it('still renders the original template/visibility/type/team chips when new props are also passed (no cross-talk)', () => {
      render(RepositorySummary, {
        props: {
          ...baseProps,
          typeName: 'library',
          teamName: 'platform-eng',
          description: 'A template for Rust libraries.',
          tags: ['rust'],
          variables: [{ name: 'service_name', value: 'my-service' }],
        },
      });
      expect(screen.getByText('Template')).toBeInTheDocument();
      expect(screen.getByText('rust-library')).toBeInTheDocument();
      expect(screen.getByText('Type')).toBeInTheDocument();
      expect(screen.getByText('library')).toBeInTheDocument();
      expect(screen.getByText('Team')).toBeInTheDocument();
      expect(screen.getByText('platform-eng')).toBeInTheDocument();
      expect(screen.getByText('Visibility')).toBeInTheDocument();
    });

    it('does not throw when rendered with only the original props (backward compatibility)', () => {
      expect(() => render(RepositorySummary, { props: baseProps })).not.toThrow();
    });
  });
});
