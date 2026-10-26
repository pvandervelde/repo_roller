<script lang="ts">
  /**
   * RepositoryTypeBadge
   *
   * Shared repository-type badge (`data-policy` + name). Extracted
   * (REFACTOR pass, create-wizard sidebar/review follow-up task) from
   * `TemplateCard.svelte`'s original `.template-card__type-badge` (CMP-004)
   * markup+CSS, which `TemplateDetailsPanel.svelte` had also copy-pasted
   * (`<span class="template-card__type-badge">`) without importing
   * TemplateCard.
   *
   * Deliberately keeps the `template-card__type-badge` class name (rather
   * than a neutral name) because `TemplateCard.test.ts`/
   * `TemplateDetailsPanel.test.ts` (frozen RED-phase contracts) assert
   * against that literal string via `querySelector`.
   *
   * Notes: fixes the same Svelte per-component style-scoping gap described
   * in `TagList.svelte`'s doc comment — `TemplateDetailsPanel`'s borrowed
   * `.template-card__type-badge` class previously rendered completely
   * unstyled (no background/color/padding) because the CSS rule lived in
   * TemplateCard's own scoped `<style>` block, which never tags markup
   * TemplateDetailsPanel renders.
   */

  interface Props {
    /**
     * Widened to `string | null | undefined` (rather than TemplateCard's
     * own stricter `string`) because this component is also consumed by
     * TemplateDetailsPanel against `GetTemplateDetailsResponse`'s
     * `RepositoryTypePolicy.type_name?: string | null` shape.
     */
    typeName?: string | null;
    /**
     * Widened to `string` (rather than TemplateCard's own stricter
     * `'fixed' | 'preferable' | 'optional'` union) for the same reason —
     * `RepositoryTypePolicy.policy` is a plain `string` on the wire. This
     * is a DOM `data-*` attribute, not a styling switch, so an unrecognised
     * value is harmless (no CSS keys off `data-policy`).
     */
    policy: string;
    /**
     * Adds a top margin before the badge. Needed in TemplateCard's block
     * layout; left off (default) when the parent already manages spacing
     * via flex `gap` (TemplateDetailsPanel).
     */
    topGap?: boolean;
  }

  let { typeName, policy, topGap = false }: Props = $props();
</script>

<span
  class="template-card__type-badge"
  class:template-card__type-badge--gap={topGap}
  data-policy={policy}
>
  {typeName}
</span>

<style>
  .template-card__type-badge {
    display: inline-block;
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    background-color: color-mix(in srgb, var(--brand-primary) 15%, transparent);
    color: var(--brand-primary);
    border-radius: 0.25rem;
  }

  .template-card__type-badge--gap {
    margin-top: 0.5rem;
  }
</style>
