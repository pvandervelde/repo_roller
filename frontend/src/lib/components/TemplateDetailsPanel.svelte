<script lang="ts">
  /**
   * TemplateDetailsPanel
   *
   * Step-1 sidebar showing full details (description, tags, repository-type
   * badge, variables) for the currently-selected template. Reuses the
   * shared `TagList` (uncapped here) and `RepositoryTypeBadge` components —
   * see those files' doc comments for the TemplateCard-originated
   * convention they share.
   *
   * Spec: docs/spec/ux/components/component-inventory.md (new component —
   * create-wizard sidebar/review follow-up task).
   */

  import Spinner from './Spinner.svelte';
  import TagList from './TagList.svelte';
  import TypeBadge from './RepositoryTypeBadge.svelte';
  import type { GetTemplateDetailsResponse } from '$lib/api/types';

  interface Props {
    templateDetails: GetTemplateDetailsResponse | null;
    loading: boolean;
    error?: string | null;
  }

  let { templateDetails, loading, error = null }: Props = $props();
</script>

<aside class="template-details-panel" aria-label="Template details">
  {#if loading}
    <div class="template-details-panel__status" role="status" aria-live="polite">
      <Spinner size="sm" />
      <span>Loading template details...</span>
    </div>
  {:else if error}
    <p class="template-details-panel__error" role="alert">{error}</p>
  {:else if templateDetails}
    <h2 class="template-details-panel__heading">{templateDetails.displayName}</h2>
    <p class="template-details-panel__description">{templateDetails.description}</p>

    {#if templateDetails.repository_type}
      <TypeBadge
        typeName={templateDetails.repository_type.type_name}
        policy={templateDetails.repository_type.policy}
      />
    {/if}

    <TagList tags={templateDetails.tags} ariaLabel="Tags" />

    {#if templateDetails.variables.length > 0}
      <ul class="template-details-panel__variables" aria-label="Template variables">
        {#each templateDetails.variables as variable (variable.name)}
          <li class="template-details-panel__variable">
            <span class="template-details-panel__variable-name">{variable.name}</span>
            <span class="template-details-panel__variable-required">
              {variable.required ? 'Required' : 'Optional'}
            </span>
            {#if variable.description}
              <p class="template-details-panel__variable-description">{variable.description}</p>
            {/if}
            {#if variable.default}
              <p class="template-details-panel__variable-default">Default: {variable.default}</p>
            {/if}
            {#if variable.pattern}
              <p class="template-details-panel__variable-pattern">Pattern: {variable.pattern}</p>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <p class="template-details-panel__empty" role="status">
      Select a template to see its details here.
    </p>
  {/if}
</aside>

<style>
  .template-details-panel {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background-color: var(--color-surface);
  }

  .template-details-panel__status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .template-details-panel__error {
    color: #b91c1c;
    font-size: 0.875rem;
    margin: 0;
  }

  .template-details-panel__empty {
    color: var(--color-text-muted);
    font-size: 0.875rem;
    margin: 0;
  }

  .template-details-panel__heading {
    font-size: 1.0625rem;
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
  }

  .template-details-panel__description {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    margin: 0;
  }

  /* Tags and the type badge are now rendered by the shared TagList/
     RepositoryTypeBadge components (see imports above) — extracted during
     this task's REFACTOR pass. Previously this component copy-pasted
     TemplateCard's `.template-card__tag` class literal into its own markup
     without importing TemplateCard, which rendered unstyled (Svelte scopes
     a component's CSS to markup that component itself renders). */

  .template-details-panel__variables {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .template-details-panel__variable {
    padding: 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .template-details-panel__variable-name {
    font-family: monospace;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .template-details-panel__variable-required {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .template-details-panel__variable-description,
  .template-details-panel__variable-default,
  .template-details-panel__variable-pattern {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin: 0;
  }
</style>
