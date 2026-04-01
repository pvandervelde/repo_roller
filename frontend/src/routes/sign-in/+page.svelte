<script lang="ts">
  import type { PageData } from './$types';
  import BrandCard from '$lib/components/BrandCard.svelte';

  const { data }: { data: PageData } = $props();

  const appName = $derived(data.brandConfig.appName);
  const logoUrl = $derived(data.brandConfig.logoUrl);
  const logoUrlDark = $derived(data.brandConfig.logoUrlDark);
  const logoAlt = $derived(data.brandConfig.logoAlt ?? null);
</script>

<svelte:head>
  <title>Sign in — {appName}</title>
</svelte:head>

<BrandCard {appName} {logoUrl} {logoUrlDark} logoAlt={logoAlt ?? undefined}>
  <h1>Sign in to {appName}</h1>
  <p class="sign-in__description">
    Create standardized GitHub repositories from your organization's templates.
  </p>
  <a href={data.githubAuthUrl} role="button" class="sign-in__button">Sign in with GitHub</a>
</BrandCard>

<style>
  .sign-in__description {
    color: var(--color-text-muted);
    margin: 0 0 1.5rem;
    font-size: 0.9375rem;
  }

  .sign-in__button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    gap: 0.625rem;
    padding: 0.625rem 1rem;
    background-color: var(--brand-primary);
    color: #ffffff;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    font-weight: 500;
    text-decoration: none;
    transition: opacity 0.15s;
  }

  .sign-in__button:hover {
    opacity: 0.88;
    text-decoration: none;
  }
</style>
