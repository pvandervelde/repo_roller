import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import RepositoryNameField from '../src/lib/components/RepositoryNameField.svelte';

// ---------------------------------------------------------------------------
// Mock API client
// ---------------------------------------------------------------------------

vi.mock('../src/lib/api/client', () => ({
  validateRepositoryName: vi.fn(),
}));

import { validateRepositoryName } from '../src/lib/api/client';

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(validateRepositoryName).mockResolvedValue({ valid: true, available: true });
});

afterEach(() => {
  vi.useRealTimers();
});

describe('RepositoryNameField (CMP-006)', () => {
  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  it('renders a labeled text input', () => {
    render(RepositoryNameField, { props: { value: '', organization: 'test-org' } });
    expect(screen.getByRole('textbox', { name: 'Repository name' })).toBeInTheDocument();
  });

  it('renders with the provided value', () => {
    render(RepositoryNameField, { props: { value: 'my-repo', organization: 'test-org' } });
    expect(screen.getByRole('textbox')).toHaveValue('my-repo');
  });

  it('renders the helper text', () => {
    render(RepositoryNameField, { props: { value: '', organization: 'test-org' } });
    expect(screen.getByText(/lowercase letters/i)).toBeInTheDocument();
  });

  it('disables the input when disabled=true', () => {
    render(RepositoryNameField, {
      props: { value: '', organization: 'test-org', disabled: true },
    });
    expect(screen.getByRole('textbox')).toBeDisabled();
  });

  // -------------------------------------------------------------------------
  // Client-side format validation — UX-ASSERT-009
  // -------------------------------------------------------------------------

  it('shows a format error after debounce for an invalid name (uppercase)', async () => {
    const onvalidationResult = vi.fn();
    render(RepositoryNameField, {
      props: { value: '', organization: 'test-org', onvalidationResult },
    });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: 'MyRepo' } });
    vi.advanceTimersByTime(300);
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
    expect(onvalidationResult).toHaveBeenCalledWith(
      expect.objectContaining({ status: 'invalid_format' }),
    );
  });

  it('shows a format error for a name starting with a dot', async () => {
    render(RepositoryNameField, { props: { value: '', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: '.hidden' } });
    vi.advanceTimersByTime(300);
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
  });

  it('shows a format error for a name containing ".."', async () => {
    render(RepositoryNameField, { props: { value: '', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: 'my..repo' } });
    vi.advanceTimersByTime(300);
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());
  });

  it('does not make an API call when the format is invalid — UX-ASSERT-009', async () => {
    render(RepositoryNameField, { props: { value: '', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: 'MyRepo' } });
    await fireEvent.blur(input);
    await vi.runAllTimersAsync();
    expect(validateRepositoryName).not.toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // Uniqueness check (blur) — UX-ASSERT-010
  // -------------------------------------------------------------------------

  it('shows "Checking availability…" during API call — UX-ASSERT-010', async () => {
    vi.mocked(validateRepositoryName).mockImplementation(
      () => new Promise(() => { }), // never resolves
    );
    render(RepositoryNameField, { props: { value: 'valid-name', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.blur(input);
    expect(screen.getByRole('status')).toHaveTextContent('Checking availability…');
  });

  it('shows "Available" when the API reports the name is valid', async () => {
    render(RepositoryNameField, { props: { value: 'valid-name', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.blur(input);
    await waitFor(() => expect(validateRepositoryName).toHaveBeenCalled());
    await waitFor(() => expect(screen.getByText('Available')).toBeInTheDocument());
  });

  it('shows "already taken" error when API reports name is taken', async () => {
    vi.mocked(validateRepositoryName).mockResolvedValue({ valid: true, available: false });
    render(RepositoryNameField, { props: { value: 'taken-name', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.blur(input);
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('is already taken'));
  });

  it('shows the check_failed warning when API throws — UX-ASSERT-028', async () => {
    vi.mocked(validateRepositoryName).mockRejectedValue(new Error('Network error'));
    render(RepositoryNameField, { props: { value: 'my-repo', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.blur(input);
    await waitFor(() =>
      expect(screen.getByRole('status')).toHaveTextContent('Could not check availability'),
    );
  });

  // -------------------------------------------------------------------------
  // Clearing "taken" on keystroke — UX-ASSERT-013
  // -------------------------------------------------------------------------

  it('clears "taken" error immediately on any keystroke — UX-ASSERT-013', async () => {
    vi.mocked(validateRepositoryName).mockResolvedValue({ valid: true, available: false });
    render(RepositoryNameField, { props: { value: 'taken-name', organization: 'test-org' } });
    const input = screen.getByRole('textbox');
    await fireEvent.blur(input);
    await waitFor(() => expect(screen.getByRole('alert')).toBeInTheDocument());

    await fireEvent.input(input, { target: { value: 'taken-names' } });
    await waitFor(() => expect(screen.queryByRole('alert')).toBeNull());
  });

  // -------------------------------------------------------------------------
  // Callbacks
  // -------------------------------------------------------------------------

  it('calls onchange with the new value on input', async () => {
    const onchange = vi.fn();
    render(RepositoryNameField, {
      props: { value: '', organization: 'test-org', onchange },
    });
    await fireEvent.input(screen.getByRole('textbox'), { target: { value: 'my-new-repo' } });
    expect(onchange).toHaveBeenCalledWith('my-new-repo');
  });

  it('calls onvalidationResult with "available" after successful check', async () => {
    const onvalidationResult = vi.fn();
    render(RepositoryNameField, {
      props: { value: 'valid-name', organization: 'test-org', onvalidationResult },
    });
    await fireEvent.blur(screen.getByRole('textbox'));
    await waitFor(() => expect(onvalidationResult).toHaveBeenCalledWith({ status: 'available' }));
  });
});
