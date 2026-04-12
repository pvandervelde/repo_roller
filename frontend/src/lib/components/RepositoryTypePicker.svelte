<script lang="ts">
  /**
   * CMP-007: RepositoryTypePicker
   *
   * Displays repository type as a read-only badge (fixed policy) or an editable
   * dropdown (preferable/optional policy).
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-007
   */

  export type RepositoryTypeOption = { name: string; description: string };

  interface Props {
    policy: 'fixed' | 'preferable' | 'optional';
    templateTypeName?: string | null;
    availableTypes: RepositoryTypeOption[];
    selectedTypeName: string | null;
    loading?: boolean;
    disabled?: boolean;
    onchange?: (typeName: string | null) => void;
  }

  let {
    policy,
    templateTypeName = null,
    availableTypes,
    selectedTypeName,
    loading = false,
    disabled = false,
    onchange,
  }: Props = $props();

  function handleChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    onchange?.(val === '' ? null : val);
  }
</script>

<div class="repo-type-picker">
  <label for="repository-type" class="repo-type-picker__label">Repository type</label>

  {#if loading}
    <p role="status" class="repo-type-picker__loading">Loading…</p>
  {:else if policy === 'fixed'}
    <div class="repo-type-picker__fixed" aria-label="Repository type: {templateTypeName}">
      <span class="repo-type-picker__lock" aria-hidden="true">🔒</span>
      <span class="repo-type-picker__fixed-value">{templateTypeName}</span>
    </div>
    <p class="repo-type-picker__helper">This template requires a specific repository type.</p>
  {:else}
    <select
      id="repository-type"
      class="repo-type-picker__select"
      {disabled}
      onchange={handleChange}
    >
      {#if policy === 'optional'}
        <option value="" selected={selectedTypeName === null}>No specific type</option>
      {/if}
      {#each availableTypes as type}
        <option value={type.name} selected={type.name === selectedTypeName}>{type.name}</option>
      {/each}
    </select>
    {#if policy === 'preferable'}
      <p class="repo-type-picker__helper">
        Recommended by this template, but you can choose a different type.
      </p>
    {/if}
  {/if}
</div>

<style>
  .repo-type-picker {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .repo-type-picker__label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .repo-type-picker__fixed {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.375rem;
    background-color: #f9fafb;
    font-size: 0.875rem;
    color: #374151;
  }

  .repo-type-picker__select {
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #111827;
    background-color: #fff;
    width: 100%;
    box-sizing: border-box;
  }

  .repo-type-picker__select:focus {
    outline: 2px solid var(--brand-primary, #2563eb);
    outline-offset: 2px;
  }

  .repo-type-picker__select:disabled {
    background-color: #f9fafb;
    color: #9ca3af;
    cursor: not-allowed;
  }

  .repo-type-picker__helper {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  .repo-type-picker__loading {
    font-size: 0.875rem;
    color: #6b7280;
    margin: 0;
  }
</style>
