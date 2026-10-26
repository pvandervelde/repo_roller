<script lang="ts">
  /**
   * CMP-004: TemplateCard
   *
   * Selectable card representing a single template. Implemented as a <label>
   * wrapping a visually-hidden <input type="radio"> for proper screen-reader
   * selection semantics and group context.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-004
   */

  import TagList from './TagList.svelte';
  import TypeBadge from './RepositoryTypeBadge.svelte';

  export type RepositoryTypeBadge = {
    typeName: string;
    policy: 'fixed' | 'preferable' | 'optional';
  } | null;

  interface Props {
    name: string;
    displayName: string;
    description: string;
    tags?: string[];
    repositoryTypeBadge?: RepositoryTypeBadge;
    selected: boolean;
    loading?: boolean;
    onselect?: () => void;
    ondeselect?: () => void;
  }

  let {
    name,
    displayName,
    description,
    tags = [],
    repositoryTypeBadge = null,
    selected,
    loading = false,
    onselect,
    ondeselect,
  }: Props = $props();

  let descriptionExpanded = $state(false);

  function toggleDescription(event: MouseEvent) {
    event.stopPropagation();
    descriptionExpanded = !descriptionExpanded;
  }
</script>

{#if loading}
  <div class="template-card template-card--loading" aria-hidden="true">
    <div class="template-card__skeleton-name"></div>
    <div class="template-card__skeleton-desc"></div>
  </div>
{:else}
  <label class="template-card" class:template-card--selected={selected}>
    <input
      type="radio"
      name="template-selection"
      value={name}
      checked={selected}
      aria-label={name}
      class="template-card__radio"
      onclick={(e) => {
        e.preventDefault();
        if (selected) {
          ondeselect?.();
        } else {
          onselect?.();
        }
      }}
    />
    {#if selected}
      <span class="template-card__checkmark" aria-hidden="true">✓</span>
    {/if}
    <h3 class="template-card__name">{displayName}</h3>
    <p
      class="template-card__description"
      class:template-card__description--expanded={descriptionExpanded}
    >
      {description}
    </p>
    <button
      type="button"
      class="template-card__description-toggle"
      aria-expanded={descriptionExpanded}
      onclick={toggleDescription}
    >
      {descriptionExpanded ? 'Show less' : 'Show more'}
    </button>
    <TagList {tags} max={4} ariaLabel="Tags" topGap />
    {#if repositoryTypeBadge}
      <TypeBadge
        typeName={repositoryTypeBadge.typeName}
        policy={repositoryTypeBadge.policy}
        topGap
      />
    {/if}
  </label>
{/if}

<style>
  .template-card {
    display: block;
    position: relative;
    padding: 1rem;
    border: 2px solid var(--color-border);
    border-radius: 0.5rem;
    cursor: pointer;
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
    background-color: var(--color-surface);
  }

  .template-card:hover {
    border-color: var(--brand-primary);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
  }

  .template-card--selected {
    border-color: var(--brand-primary);
    box-shadow: 0 0 0 2px var(--brand-primary);
  }

  /* Visually hidden but accessible */
  .template-card__radio {
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

  .template-card__checkmark {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    color: var(--brand-primary, #2563eb);
    font-size: 1.125rem;
    font-weight: bold;
  }

  .template-card__name {
    font-size: 0.9375rem;
    font-weight: 600;
    margin: 0 0 0.375rem;
    color: var(--color-text);
    padding-right: 1.5rem;
  }

  .template-card__description {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    margin: 0 0 0.5rem;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .template-card__description--expanded {
    display: block;
    -webkit-line-clamp: unset;
    line-clamp: unset;
    overflow: visible;
  }

  .template-card__description-toggle {
    display: inline-block;
    padding: 0;
    margin: 0 0 0.5rem;
    border: none;
    background: none;
    color: var(--brand-primary);
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
  }

  .template-card__description-toggle:focus-visible {
    outline: 2px solid var(--focus-ring);
    outline-offset: 2px;
  }

  /* Tag chips and the repository-type badge are now owned by the shared
     TagList/RepositoryTypeBadge components (see imports above) — extracted
     during the create-wizard sidebar/review follow-up task's REFACTOR pass
     to remove duplication against RepositorySummary/TemplateDetailsPanel,
     which had each independently reimplemented or copy-pasted this CSS. */

  /* Loading skeleton */
  .template-card--loading {
    cursor: default;
    background-color: var(--color-surface);
    border-color: var(--color-border);
  }

  .template-card__skeleton-name {
    height: 1rem;
    background-color: var(--color-border);
    border-radius: 0.25rem;
    margin-bottom: 0.5rem;
    width: 60%;
    animation: skeleton-pulse 1.5s ease-in-out infinite;
  }

  .template-card__skeleton-desc {
    height: 0.875rem;
    background-color: var(--color-border);
    border-radius: 0.25rem;
    width: 85%;
    animation: skeleton-pulse 1.5s ease-in-out infinite;
  }

  @keyframes skeleton-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
