<script lang="ts">
  /**
   * CMP-009: RepositorySummary
   *
   * Read-only summary of the repository about to be (or just) created.
   * Shows "[org] / [name]" with template, type, visibility, and team chips.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-009
   */

  interface Props {
    organization: string;
    repositoryName: string;
    templateName: string;
    typeName?: string | null;
    visibility: 'private' | 'public';
    teamName?: string | null;
  }

  let {
    organization,
    repositoryName,
    templateName,
    typeName = null,
    visibility,
    teamName = null,
  }: Props = $props();

  const displayName = $derived(repositoryName || '—');
</script>

<div class="repo-summary" aria-label="Repository summary">
  <p class="repo-summary__prefix">You are about to create:</p>
  <p class="repo-summary__name">
    <span class="repo-summary__org">{organization}</span>
    <span class="repo-summary__separator" aria-hidden="true"> / </span>
    <span class="repo-summary__repo" class:repo-summary__repo--placeholder={!repositoryName}>
      {displayName}
    </span>
  </p>
  <ul class="repo-summary__chips" aria-label="Repository configuration">
    <li class="repo-summary__chip">
      <span class="repo-summary__chip-label">Template</span>
      <span class="repo-summary__chip-value">{templateName}</span>
    </li>
    {#if typeName}
      <li class="repo-summary__chip">
        <span class="repo-summary__chip-label">Type</span>
        <span class="repo-summary__chip-value">{typeName}</span>
      </li>
    {/if}
    <li class="repo-summary__chip">
      <span class="repo-summary__chip-label">Visibility</span>
      <span class="repo-summary__chip-value">{visibility}</span>
    </li>
    {#if teamName}
      <li class="repo-summary__chip">
        <span class="repo-summary__chip-label">Team</span>
        <span class="repo-summary__chip-value">{teamName}</span>
      </li>
    {/if}
  </ul>
</div>

<style>
  .repo-summary {
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    padding: 1rem;
    background-color: #f9fafb;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .repo-summary__prefix {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  .repo-summary__name {
    font-size: 1rem;
    font-weight: 600;
    font-family: monospace;
    color: #111827;
    margin: 0;
  }

  .repo-summary__repo--placeholder {
    color: #9ca3af;
    font-style: italic;
  }

  .repo-summary__chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .repo-summary__chip {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.625rem;
    background-color: #e5e7eb;
    border-radius: 9999px;
    font-size: 0.75rem;
  }

  .repo-summary__chip-label {
    color: #6b7280;
    font-weight: 400;
  }

  .repo-summary__chip-value {
    color: #111827;
    font-weight: 500;
  }
</style>
