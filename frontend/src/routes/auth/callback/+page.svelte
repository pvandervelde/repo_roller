<script lang="ts">
  import type { BrandConfig } from '$lib/types/brand';
  import BrandCard from '$lib/components/BrandCard.svelte';

  // The callback load always redirects; this page briefly shows during OAuth processing.
  // brandConfig is provided by the root layout's load function via SvelteKit's data merging.
  const { data }: { data: { brandConfig: BrandConfig; session: unknown } } = $props();

  const appName = $derived(data.brandConfig.appName);
  const logoUrl = $derived(data.brandConfig.logoUrl);
  const logoUrlDark = $derived(data.brandConfig.logoUrlDark);
  const logoAlt = $derived(data.brandConfig.logoAlt);
</script>

<svelte:head>
  <title>Signing in… — {appName}</title>
</svelte:head>

<BrandCard {appName} {logoUrl} {logoUrlDark} logoAlt={logoAlt ?? undefined}>
  <h1>Completing sign-in</h1>
  <div class="callback__spinner" role="status" aria-label="Signing in">
    <span class="callback__spinner-icon" aria-hidden="true"></span>
  </div>
  <p class="callback__message">You'll be redirected in a moment.</p>
</BrandCard>

<style>
  .callback__spinner {
    display: flex;
    justify-content: center;
    margin: 1.5rem 0 1rem;
  }

  .callback__spinner-icon {
    display: inline-block;
    width: 2rem;
    height: 2rem;
    border: 3px solid var(--color-border);
    border-top-color: var(--brand-primary);
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .callback__message {
    text-align: center;
    color: var(--color-text-muted);
    font-size: 0.9375rem;
    margin: 0;
  }
</style>
