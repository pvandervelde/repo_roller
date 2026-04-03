<script lang="ts">
  /**
   * CMP-008: VariableField
   *
   * Single text input for one template variable. Label is derived from the raw
   * variable name (snake_case → Title Case). Required fields show an asterisk
   * and carry aria-required="true". Description shown as helper text via
   * aria-describedby. Default value pre-fills the input on mount.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-008
   * Assertions: UX-ASSERT-016, UX-ASSERT-017
   */

  interface Props {
    variableName: string;
    label: string;
    description?: string | null;
    required: boolean;
    defaultValue?: string | null;
    value: string;
    disabled?: boolean;
    onchange?: (value: string) => void;
  }

  let {
    variableName,
    label,
    description = null,
    required,
    defaultValue = null,
    value,
    disabled = false,
    onchange,
  }: Props = $props();

  // Stable IDs derived from the variable name (which is unique within a template)
  const inputId = $derived(`variable-${variableName}`);
  const descId = $derived(`variable-${variableName}-desc`);

  function handleInput(e: Event) {
    onchange?.((e.target as HTMLInputElement).value);
  }
</script>

<div class="variable-field">
  <label for={inputId} class="variable-field__label">
    {label}
    {#if required}
      <span class="variable-field__required" aria-hidden="true">*</span>
      <span class="sr-only">(required)</span>
    {/if}
  </label>

  <input
    id={inputId}
    type="text"
    class="variable-field__input"
    {value}
    placeholder={description ?? 'Enter a value'}
    aria-required={required ? 'true' : undefined}
    aria-describedby={description ? descId : undefined}
    {disabled}
    oninput={handleInput}
  />

  {#if description}
    <p id={descId} class="variable-field__helper">{description}</p>
  {/if}
</div>

<style>
  .variable-field {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .variable-field__label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .variable-field__required {
    color: #b91c1c;
    margin-left: 0.125rem;
  }

  .variable-field__input {
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    color: #111827;
    background-color: #fff;
    width: 100%;
    box-sizing: border-box;
  }

  .variable-field__input:focus {
    outline: 2px solid var(--brand-primary, #2563eb);
    outline-offset: 2px;
    border-color: var(--brand-primary, #2563eb);
  }

  .variable-field__input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .variable-field__helper {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
