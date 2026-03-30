<script lang="ts">
  /**
   * CMP-010: InlineAlert
   *
   * Non-modal status message. Uses role="alert" for error/warning (immediate
   * announcement) and role="status" for info/success (polite announcement).
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-010
   * Assertion: UX-ASSERT-025
   */
  interface ActionProp {
    label: string;
    onClick: () => void;
  }

  interface Props {
    variant: 'error' | 'warning' | 'info' | 'success';
    message: string;
    action?: ActionProp | null;
  }

  let { variant, message, action = null }: Props = $props();

  const role = $derived(variant === 'error' || variant === 'warning' ? 'alert' : 'status');
</script>

<div {role} data-variant={variant} class="inline-alert">
  <span class="inline-alert__message">{message}</span>
  {#if action}
    <button type="button" class="inline-alert__action" onclick={action.onClick}>
      {action.label}
    </button>
  {/if}
</div>

<style>
  .inline-alert {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-radius: 0.25rem;
    border-left: 4px solid currentColor;
  }

  .inline-alert[data-variant='error'] {
    background-color: #fef2f2;
    color: #b91c1c;
  }

  .inline-alert[data-variant='warning'] {
    background-color: #fffbeb;
    color: #b45309;
  }

  .inline-alert[data-variant='info'] {
    background-color: #eff6ff;
    color: var(--brand-primary, #2563eb);
  }

  .inline-alert[data-variant='success'] {
    background-color: #f0fdf4;
    color: #15803d;
  }

  .inline-alert__message {
    flex: 1;
  }

  .inline-alert__action {
    flex-shrink: 0;
    background: none;
    border: 1px solid currentColor;
    border-radius: 0.25rem;
    padding: 0.25rem 0.75rem;
    color: inherit;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .inline-alert__action:hover {
    opacity: 0.8;
  }
</style>
