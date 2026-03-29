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
</script>

<!-- TODO: implement -->
