<script lang="ts">
  /**
   * TagList
   *
   * Shared tag-chip list. Extracted (REFACTOR pass, create-wizard
   * sidebar/review follow-up task) from markup+CSS that had been
   * independently duplicated three times: `TemplateCard.svelte`'s original
   * `.template-card__tags`/`.template-card__tag` (CMP-004, capped at `max`
   * tags), `TemplateDetailsPanel.svelte`'s uncapped tags list (which had
   * copy-pasted the `<li class="template-card__tag">` markup without
   * importing TemplateCard, so the CSS never actually applied — see Notes),
   * and `RepositorySummary.svelte`'s new uncapped tags list (which had
   * reimplemented the identical chip CSS under its own
   * `.repo-summary__tags`/`.repo-summary__tag` names).
   *
   * Deliberately keeps the `template-card__tags`/`template-card__tag` class
   * names (rather than a neutral `tag-list` name) because
   * `TemplateGrid.test.ts`/`TemplateCard.test.ts`/`TemplateDetailsPanel.test.ts`
   * (frozen RED-phase contracts, out of this refactor's scope to edit) assert
   * against those literal strings — renaming would make at least one
   * assertion (TemplateGrid's "renders no tags for an empty array" check)
   * vacuously pass instead of actually verifying absence.
   *
   * Renders nothing when there are no tags to show (after an optional `max`
   * cap is applied).
   *
   * Notes: Svelte scopes a component's `<style>` block to markup rendered by
   * THAT component only — a class literal borrowed from another component's
   * stylesheet (e.g. the pre-refactor `TemplateDetailsPanel` reusing
   * `.template-card__tag`) renders structurally correct but visually
   * unstyled, since the scoping hash class is only ever applied to elements
   * TemplateCard itself renders. Routing every caller through this shared
   * component (which owns its own `<style>` block) fixes that silently for
   * all three call sites.
   */

  interface Props {
    tags: string[];
    ariaLabel: string;
    max?: number;
    /**
     * Adds a top margin before the list. Needed in TemplateCard's block
     * layout (manual margins between stacked children); left off (default)
     * when the parent already manages spacing via flex `gap` (RepositorySummary,
     * TemplateDetailsPanel).
     */
    topGap?: boolean;
  }

  let { tags, ariaLabel, max, topGap = false }: Props = $props();

  const visibleTags = $derived(max !== undefined ? tags.slice(0, max) : tags);
</script>

{#if visibleTags.length > 0}
  <ul class="template-card__tags" class:template-card__tags--gap={topGap} aria-label={ariaLabel}>
    {#each visibleTags as tag (tag)}
      <li class="template-card__tag">{tag}</li>
    {/each}
  </ul>
{/if}

<style>
  .template-card__tags {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .template-card__tags--gap {
    margin-top: 0.5rem;
  }

  .template-card__tag {
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    background-color: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 9999px;
    color: var(--color-text-muted);
  }
</style>
