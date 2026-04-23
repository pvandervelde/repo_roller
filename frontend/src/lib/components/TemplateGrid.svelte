<script lang="ts">
  /**
   * CMP-005: TemplateGrid
   *
   * Grid of selectable TemplateCards with search filtering and loading/error
   * states. Radio group semantics with client-side search filtering.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-005
   */

  import type { TemplateSummary } from '$lib/api/types';
  import TemplateCard from './TemplateCard.svelte';
  import InlineAlert from './InlineAlert.svelte';

  interface Props {
    templates: TemplateSummary[];
    selectedTemplateName?: string | null;
    loading?: boolean;
    error?: string | null;
    ontemplateSelect?: (templateName: string) => void;
    ontemplateDeselect?: () => void;
    onretry?: () => void;
  }

  let {
    templates,
    selectedTemplateName = null,
    loading = false,
    error = null,
    ontemplateSelect,
    ontemplateDeselect,
    onretry,
  }: Props = $props();

  let searchQuery = $state('');

  const filteredTemplates = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return templates;
    return templates.filter(
      (t) =>
        t.name.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q) ||
        (t.category?.toLowerCase().includes(q) ?? false),
    );
  });
</script>

<div class="template-grid">
  {#if !loading}
    <div class="template-grid__search">
      <label for="template-search" class="template-grid__search-label">Search templates</label>
      <input
        id="template-search"
        type="search"
        class="template-grid__search-input"
        placeholder="Search templates"
        bind:value={searchQuery}
      />
    </div>
  {/if}

  {#if error}
    <InlineAlert
      variant="error"
      message={error}
      action={{ label: 'Try again', onClick: () => onretry?.() }}
    />
  {:else if loading}
    <p role="status" aria-live="polite" class="template-grid__loading-status">Loading templates…</p>
    <div class="template-grid__cards template-grid__cards--loading">
      {#each [0, 1, 2, 3, 4, 5, 6, 7] as _}
        <TemplateCard name="" description="" selected={false} loading={true} />
      {/each}
    </div>
  {:else if templates.length === 0}
    <p role="status" class="template-grid__empty">
      No templates are configured for this organization. Contact your platform team.
    </p>
  {:else}
    <div role="radiogroup" aria-label="Available templates" class="template-grid__cards">
      {#if filteredTemplates.length === 0}
        <p role="status" class="template-grid__no-results">
          No templates match '{searchQuery}'
        </p>
      {:else}
        {#each filteredTemplates as template}
          <TemplateCard
            name={template.name}
            description={template.description}
            tags={template.category ? [template.category] : []}
            repositoryTypeBadge={null}
            selected={template.name === selectedTemplateName}
            onselect={() => ontemplateSelect?.(template.name)}
            ondeselect={() => ontemplateDeselect?.()}
          />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .template-grid {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .template-grid__search {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .template-grid__search-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text);
  }

  .template-grid__search-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: var(--color-text);
    background-color: var(--color-bg);
    box-sizing: border-box;
  }

  .template-grid__search-input:focus {
    outline: 2px solid var(--brand-primary);
    outline-offset: 2px;
    border-color: var(--brand-primary);
  }

  .template-grid__cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
    gap: 1rem;
  }

  .template-grid__empty,
  .template-grid__no-results {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    text-align: center;
    padding: 2rem 1rem;
  }

  .template-grid__loading-status {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    text-align: center;
    padding: 0.25rem 0;
    margin: 0;
  }
</style>
