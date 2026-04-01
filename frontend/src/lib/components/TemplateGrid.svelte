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

  interface Props {
    templates: TemplateSummary[];
    selectedTemplateName?: string | null;
    loading?: boolean;
    error?: string | null;
    ontemplateSelect?: (templateName: string) => void;
    onretry?: () => void;
  }

  let {
    templates,
    selectedTemplateName = null,
    loading = false,
    error = null,
    ontemplateSelect,
    onretry,
  }: Props = $props();
</script>

<div class="template-grid"></div>
