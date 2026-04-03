import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import VariableField from '../src/lib/components/VariableField.svelte';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

interface VariableFieldProps {
  variableName: string;
  label: string;
  description?: string | null;
  required: boolean;
  defaultValue?: string | null;
  value: string;
  disabled?: boolean;
  onchange?: (value: string) => void;
}

function makeProps(overrides: Partial<VariableFieldProps> = {}): VariableFieldProps {
  return {
    variableName: 'service_name',
    label: 'Service name',
    required: false,
    value: '',
    ...overrides,
  };
}

describe('VariableField (CMP-008)', () => {
  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  it('renders a labelled text input', () => {
    render(VariableField, { props: makeProps() });
    const input = screen.getByRole('textbox', { name: /service name/i });
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('type', 'text');
  });

  it('associates the label with the input via id', () => {
    render(VariableField, { props: makeProps() });
    const input = screen.getByLabelText(/service name/i);
    expect(input).toHaveAttribute('id', 'variable-service_name');
  });

  it('renders description as helper text', () => {
    render(VariableField, {
      props: makeProps({ description: 'The name of the microservice.' }),
    });
    expect(screen.getByText('The name of the microservice.')).toBeInTheDocument();
  });

  it('does not render helper text when description is null', () => {
    render(VariableField, { props: makeProps({ description: null }) });
    expect(screen.queryByRole('paragraph')).toBeNull();
  });

  it('uses description as placeholder when provided', () => {
    render(VariableField, {
      props: makeProps({ description: 'The name of the microservice.' }),
    });
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('placeholder', 'The name of the microservice.');
  });

  it('uses "Enter a value" as placeholder when no description', () => {
    render(VariableField, { props: makeProps({ description: null }) });
    expect(screen.getByRole('textbox')).toHaveAttribute('placeholder', 'Enter a value');
  });

  // -------------------------------------------------------------------------
  // Required fields
  // -------------------------------------------------------------------------

  it('shows asterisk for required fields', () => {
    render(VariableField, { props: makeProps({ required: true }) });
    expect(screen.getByText('*')).toBeInTheDocument();
  });

  it('sets aria-required="true" for required fields', () => {
    render(VariableField, { props: makeProps({ required: true }) });
    expect(screen.getByRole('textbox')).toHaveAttribute('aria-required', 'true');
  });

  it('does not set aria-required for optional fields', () => {
    render(VariableField, { props: makeProps({ required: false }) });
    expect(screen.getByRole('textbox')).not.toHaveAttribute('aria-required');
  });

  it('includes "(required)" sr-only text for required fields', () => {
    render(VariableField, { props: makeProps({ required: true }) });
    // The sr-only span should be present in the document (screen readers announce it)
    expect(screen.getByText('(required)')).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Accessibility — aria-describedby
  // -------------------------------------------------------------------------

  it('sets aria-describedby when description is present', () => {
    render(VariableField, {
      props: makeProps({ description: 'Some help text.' }),
    });
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('aria-describedby', 'variable-service_name-desc');
  });

  it('does not set aria-describedby when description is null', () => {
    render(VariableField, { props: makeProps({ description: null }) });
    expect(screen.getByRole('textbox')).not.toHaveAttribute('aria-describedby');
  });

  // -------------------------------------------------------------------------
  // Default value pre-fill (UX-ASSERT-017)
  // -------------------------------------------------------------------------

  it('pre-fills the input with the provided value', () => {
    render(VariableField, {
      props: makeProps({ value: 'my-default', defaultValue: 'my-default' }),
    });
    expect(screen.getByRole('textbox')).toHaveValue('my-default');
  });

  it('shows empty input when value is empty string', () => {
    render(VariableField, { props: makeProps({ value: '' }) });
    expect(screen.getByRole('textbox')).toHaveValue('');
  });

  // -------------------------------------------------------------------------
  // Disabled state
  // -------------------------------------------------------------------------

  it('disables the input when disabled prop is true', () => {
    render(VariableField, { props: makeProps({ disabled: true }) });
    expect(screen.getByRole('textbox')).toBeDisabled();
  });

  // -------------------------------------------------------------------------
  // onchange callback
  // -------------------------------------------------------------------------

  it('calls onchange with new value when user types', async () => {
    const onchange = vi.fn();
    render(VariableField, { props: makeProps({ onchange }) });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: 'hello' } });
    expect(onchange).toHaveBeenCalledWith('hello');
  });

  it('calls onchange with empty string when user clears the field', async () => {
    const onchange = vi.fn();
    render(VariableField, { props: makeProps({ value: 'hello', onchange }) });
    const input = screen.getByRole('textbox');
    await fireEvent.input(input, { target: { value: '' } });
    expect(onchange).toHaveBeenCalledWith('');
  });
});
