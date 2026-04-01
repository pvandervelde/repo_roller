<script lang="ts">
  import type { PageData } from './$types';
  import BrandCard from '$lib/components/BrandCard.svelte';

  const { data }: { data: PageData } = $props();

  const appName = $derived(data.brandConfig.appName);
  const logoUrl = $derived(data.brandConfig.logoUrl);
  const logoUrlDark = $derived(data.brandConfig.logoUrlDark);
  const logoAlt = $derived(data.brandConfig.logoAlt ?? null);

  type Variant = 'oauth_error' | 'access_denied';

  const variant = $derived<Variant>(
    data.reason === 'access_denied' ? 'access_denied' : 'oauth_error',
  );

  const heading = $derived(
    variant === 'access_denied'
      ? 'GitHub authorization was cancelled'
      : 'Sign-in could not be completed',
  );

  const message = $derived(
    variant === 'access_denied'
      ? `${appName} needs permission to read your GitHub identity to log who creates repositories.`
      : 'There was a problem connecting to GitHub. This is usually temporary.',
  );
</script>

<svelte:head>
  <title>Access denied — {appName}</title>
</svelte:head>

<BrandCard {appName} {logoUrl} {logoUrlDark} logoAlt={logoAlt ?? undefined}>
  <h1>{heading}</h1>
  <p class="denied__message" role="alert">{message}</p>
  <a href="/sign-in" class="denied__button">Try again</a>
</BrandCard>

<style>
  .denied__message {
    color: var(--color-text-muted);
    font-size: 0.9375rem;
    margin: 0 0 1.5rem;
  }

  .denied__button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    padding: 0.625rem 1rem;
    background-color: var(--brand-primary);
    color: #ffffff;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    font-weight: 500;
    text-decoration: none;
    transition: opacity 0.15s;
  }

  .denied__button:hover {
    opacity: 0.88;
    text-decoration: none;
  }
</style>
