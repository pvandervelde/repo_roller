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
});
