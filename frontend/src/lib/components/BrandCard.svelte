<script lang="ts">
  /**
   * CMP-001b: BrandCard
   *
   * Centred card shell for unauthenticated screens (sign-in, OAuth callback, access denied).
   * Displays brand logo/wordmark above the card; no header or footer.
   * Logo rendering: identical logic to AppShell — <picture>, <img>, or text wordmark.
   * appName text is always present in the DOM (sr-only when logo is shown) — UX-ASSERT-026.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-001b
   * Assertions: UX-ASSERT-026, UX-ASSERT-027
   */
  import type { Snippet } from 'svelte';

  interface Props {
    appName: string;
    logoUrl?: string | null;
    logoUrlDark?: string | null;
    logoAlt?: string;
    children: Snippet;
  }

  let { appName, logoUrl = null, logoUrlDark = null, logoAlt, children }: Props = $props();

  const resolvedLogoAlt = $derived(logoAlt ?? `${appName} logo`);
</script>

<!-- TODO: implement -->
