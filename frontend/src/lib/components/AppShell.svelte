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

<header class="app-shell__header">
  <a href="/" class="app-shell__brand">
    {#if logoUrl}
      {#if logoUrlDark}
        <picture>
          <source media="(prefers-color-scheme: dark)" srcset={logoUrlDark} />
          <img src={logoUrl} alt={resolvedLogoAlt} class="app-shell__logo" />
        </picture>
      {:else}
        <img src={logoUrl} alt={resolvedLogoAlt} class="app-shell__logo" />
      {/if}
      <span class="sr-only">{appName}</span>
    {:else}
      <span class="app-shell__wordmark">{appName}</span>
    {/if}
  </a>
  {#if userLogin}
    <UserBadge login={userLogin} avatarUrl={userAvatarUrl} {onsignOut} />
  {/if}
</header>

<main class="app-shell__main">
  {@render children()}
</main>

<style>
  .app-shell__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
    height: 3.5rem;
    background-color: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .app-shell__brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    text-decoration: none;
    color: inherit;
  }

  .app-shell__logo {
    height: 1.75rem;
    width: auto;
  }

  .app-shell__wordmark {
    font-size: 1rem;
    font-weight: 600;
    color: var(--brand-primary);
    letter-spacing: -0.01em;
  }

  .app-shell__main {
    padding: 2rem 1.5rem;
    max-width: 72rem;
    margin: 0 auto;
    width: 100%;
  }
</style>
