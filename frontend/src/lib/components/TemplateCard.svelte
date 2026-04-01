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

  export type RepositoryTypeBadge = {
    typeName: string;
    policy: 'fixed' | 'preferable' | 'optional';
  } | null;

  interface Props {
    name: string;
    description: string;
    tags?: string[];
    repositoryTypeBadge?: RepositoryTypeBadge;
    selected: boolean;
    loading?: boolean;
    onselect?: () => void;
  }

  let {
    name,
    description,
    tags = [],
    repositoryTypeBadge = null,
    selected,
    loading = false,
    onselect,
  }: Props = $props();
</script>

<div class="template-card"></div>
