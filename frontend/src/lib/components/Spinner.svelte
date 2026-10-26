<script lang="ts">
  /**
   * CMP-012: Spinner
   *
   * Shared loading spinner, extracted from CreationOverlay's original
   * `.creation-overlay__spinner` + `@keyframes spin` (0.8s linear infinite).
   * Used both standalone (role="status", self-announcing via `label`) and
   * nested inside a parent-owned aria-live region (label omitted, so the
   * spinner itself stays out of the accessibility tree via aria-hidden).
   *
   * Spec: docs/spec/ux/components/component-inventory.md (new — extracted
   * from CMP-011 per the create-wizard sidebar/review follow-up task).
   */

  interface Props {
    size?: 'sm' | 'md' | 'lg';
    label?: string;
  }

  let { size = 'md', label }: Props = $props();
</script>

{#if label}
  <div class="spinner spinner--{size}" role="status" aria-label={label}></div>
{:else}
  <div class="spinner spinner--{size}" aria-hidden="true"></div>
{/if}

<style>
  .spinner {
    border: 4px solid var(--color-border);
    border-top-color: var(--brand-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner--sm {
    width: 1.25rem;
    height: 1.25rem;
    border-width: 2px;
  }

  .spinner--md {
    width: 2rem;
    height: 2rem;
    border-width: 3px;
  }

  .spinner--lg {
    width: 3rem;
    height: 3rem;
    border-width: 4px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
