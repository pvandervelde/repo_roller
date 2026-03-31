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

<div class="brand-card__wrapper">
  <div class="brand-card__brand">
    {#if logoUrl}
      {#if logoUrlDark}
        <picture>
          <source media="(prefers-color-scheme: dark)" srcset={logoUrlDark} />
          <img src={logoUrl} alt={resolvedLogoAlt} class="brand-card__logo" />
        </picture>
      {:else}
        <img src={logoUrl} alt={resolvedLogoAlt} class="brand-card__logo" />
      {/if}
      <span class="sr-only">{appName}</span>
    {:else}
      <span class="brand-card__wordmark">{appName}</span>
    {/if}
  </div>
  <div class="brand-card__card">
    {@render children()}
  </div>
</div>

<style>
  .brand-card__wrapper {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem 1rem;
    background-color: var(--color-bg);
  }

  .brand-card__brand {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .brand-card__logo {
    height: 2.5rem;
    width: auto;
  }

  .brand-card__wordmark {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--brand-primary);
    letter-spacing: -0.02em;
  }

  .brand-card__card {
    width: 100%;
    max-width: 24rem;
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    padding: 2rem;
  }
</style>
