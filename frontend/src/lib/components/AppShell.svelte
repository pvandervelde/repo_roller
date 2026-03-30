<script lang="ts">
  /**
   * CMP-001: AppShell
   *
   * Global page layout shell with header containing brand identity and user identity.
   * Renders <header> and <main> landmarks; embeds UserBadge; forwards onsignOut.
   * Logo rendering: <picture> when both logoUrl and logoUrlDark set,
   * <img> when only logoUrl, text wordmark when logoUrl is null.
   * appName text is always present in the DOM (sr-only when logo is shown) — UX-ASSERT-026.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-001
   * Assertions: UX-ASSERT-026, UX-ASSERT-027
   */
  import type { Snippet } from 'svelte';
  import UserBadge from './UserBadge.svelte';

  interface Props {
    appName: string;
    logoUrl?: string | null;
    logoUrlDark?: string | null;
    logoAlt?: string;
    userLogin: string;
    userAvatarUrl?: string | null;
    onsignOut?: () => void;
    children: Snippet;
  }

  let {
    appName,
    logoUrl = null,
    logoUrlDark = null,
    logoAlt,
    userLogin,
    userAvatarUrl = null,
    onsignOut,
    children,
  }: Props = $props();

  const resolvedLogoAlt = $derived(logoAlt ?? `${appName} logo`);
</script>

<!-- TODO: implement -->
