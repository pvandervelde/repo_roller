import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import RepositoryTypePicker from '../src/lib/components/RepositoryTypePicker.svelte';
import type { RepositoryTypeOption } from '../src/lib/components/RepositoryTypePicker.svelte';

const types: RepositoryTypeOption[] = [
  { name: 'library', description: 'A reusable library' },
  { name: 'service', description: 'A microservice' },
  { name: 'cli', description: 'A command line tool' },
];

describe('RepositoryTypePicker (CMP-007)', () => {
  // -------------------------------------------------------------------------
  // Fixed policy
  // -------------------------------------------------------------------------

  it('renders a read-only row (no select) for fixed policy', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'fixed',
        templateTypeName: 'library',
        availableTypes: types,
        selectedTypeName: 'library',
      },
    });
    expect(screen.queryByRole('combobox')).toBeNull();
    expect(screen.getByText('library')).toBeInTheDocument();
  });

  it('shows the fixed-policy helper text', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'fixed',
        templateTypeName: 'library',
        availableTypes: types,
        selectedTypeName: 'library',
      },
    });
    expect(
      screen.getByText('This template requires a specific repository type.'),
    ).toBeInTheDocument();
  });

  it('shows a lock icon for fixed policy', () => {
    const { container } = render(RepositoryTypePicker, {
      props: {
        policy: 'fixed',
        templateTypeName: 'service',
        availableTypes: types,
        selectedTypeName: 'service',
      },
    });
    expect(container.querySelector('.repo-type-picker__lock')).toBeTruthy();
  });

  // -------------------------------------------------------------------------
  // Preferable policy
  // -------------------------------------------------------------------------

  it('renders a dropdown for preferable policy', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'preferable',
        templateTypeName: 'library',
        availableTypes: types,
        selectedTypeName: 'library',
      },
    });
    expect(screen.getByRole('combobox')).toBeInTheDocument();
  });

  it('does not include "No specific type" option for preferable policy', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'preferable',
        templateTypeName: 'library',
        availableTypes: types,
        selectedTypeName: 'library',
      },
    });
    expect(screen.queryByRole('option', { name: 'No specific type' })).toBeNull();
  });

  it('shows preferable helper text', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'preferable',
        templateTypeName: 'library',
        availableTypes: types,
        selectedTypeName: 'library',
      },
    });
    expect(
      screen.getByText('Recommended by this template, but you can choose a different type.'),
    ).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Optional policy
  // -------------------------------------------------------------------------

  it('renders a dropdown for optional policy', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'optional',
        availableTypes: types,
        selectedTypeName: null,
      },
    });
    expect(screen.getByRole('combobox')).toBeInTheDocument();
  });

  it('includes "No specific type" as the first option for optional policy', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'optional',
        availableTypes: types,
        selectedTypeName: null,
      },
    });
    const options = screen.getAllByRole('option');
    expect(options[0]).toHaveTextContent('No specific type');
  });

  // -------------------------------------------------------------------------
  // Loading state
  // -------------------------------------------------------------------------

  it('shows loading indicator when loading=true', () => {
    render(RepositoryTypePicker, {
      props: {
        policy: 'optional',
        availableTypes: [],
        selectedTypeName: null,
        loading: true,
      },
    });
    expect(screen.getByRole('status')).toBeInTheDocument();
    expect(screen.queryByRole('combobox')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Interaction
  // -------------------------------------------------------------------------

  it('calls onchange with the selected type name when the user changes the dropdown', async () => {
    const onchange = vi.fn();
    render(RepositoryTypePicker, {
      props: {
        policy: 'optional',
        availableTypes: types,
        selectedTypeName: null,
        onchange,
      },
    });
    await fireEvent.change(screen.getByRole('combobox'), { target: { value: 'service' } });
    expect(onchange).toHaveBeenCalledWith('service');
  });

  it('calls onchange with null when "No specific type" is selected', async () => {
    const onchange = vi.fn();
    render(RepositoryTypePicker, {
      props: {
        policy: 'optional',
        availableTypes: types,
        selectedTypeName: 'library',
        onchange,
      },
    });
    await fireEvent.change(screen.getByRole('combobox'), { target: { value: '' } });
    expect(onchange).toHaveBeenCalledWith(null);
  });
});
