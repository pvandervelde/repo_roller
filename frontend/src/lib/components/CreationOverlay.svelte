<script lang="ts">
  /**
   * CMP-011: CreationOverlay
   *
   * Full-area overlay displayed while POST /api/v1/repositories is in flight.
   * When visible: wizard content behind it carries aria-hidden="true" (handled
   * by the parent via the `visible` prop) and this overlay announces its status
   * via aria-live="polite".
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-011
   * Assertions: UX-ASSERT-018
   * Copy: docs/spec/ux/copy.md — Creation overlay
   */

  interface Props {
    visible: boolean;
  }

  let { visible }: Props = $props();
</script>

{#if visible}
  <div class="creation-overlay" role="dialog" aria-modal="true" aria-label="Creating repository">
    <div class="creation-overlay__content" aria-live="polite">
      <div class="creation-overlay__spinner" role="status" aria-label="Creating repository"></div>
      <h2 class="creation-overlay__heading">Creating your repository…</h2>
      <p class="creation-overlay__message">
        This may take up to a minute. Please don't close this page.
      </p>
    </div>
  </div>
{/if}

<style>
  .creation-overlay {
    position: fixed;
    inset: 0;
    background-color: color-mix(in srgb, var(--color-bg) 92%, transparent);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .creation-overlay__content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    text-align: center;
    max-width: 24rem;
    padding: 2rem;
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 0.75rem;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.24);
  }

  .creation-overlay__spinner {
    width: 3rem;
    height: 3rem;
    border: 4px solid var(--color-border);
    border-top-color: var(--brand-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .creation-overlay__heading {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
  }

  .creation-overlay__message {
    font-size: 0.9375rem;
    color: var(--color-text-muted);
    margin: 0;
  }
</style>
