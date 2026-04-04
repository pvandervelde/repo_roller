<script lang="ts">
  import type { PageData } from './$types';
  import AppShell from '$lib/components/AppShell.svelte';

  const { data }: { data: PageData } = $props();

  const isSessionExpired = $derived(data.reason === 'session_expired');

  const heading = $derived(isSessionExpired ? 'Your session has expired' : 'Something went wrong');

  const message = $derived(
    isSessionExpired
      ? 'Please sign in again to continue.'
      : 'An unexpected error occurred. If this keeps happening, contact your platform team.',
  );

  const buttonLabel = $derived(isSessionExpired ? 'Sign in' : 'Try again');

  const buttonHref = $derived(
    isSessionExpired ? '/sign-in' : data.session ? '/create' : '/sign-in',
  );

  const icon = $derived(isSessionExpired ? '🔒' : '⚠');
</script>

<svelte:head>
  <title>Error — {data.brandConfig.appName}</title>
</svelte:head>

<AppShell
  appName={data.brandConfig.appName}
  logoUrl={data.brandConfig.logoUrl}
  logoUrlDark={data.brandConfig.logoUrlDark}
  logoAlt={data.brandConfig.logoAlt}
  userLogin={data.session?.userLogin ?? ''}
  userAvatarUrl={data.session?.userAvatarUrl ?? null}
>
  <div class="error-screen">
    <span class="error-screen__icon" aria-hidden="true">{icon}</span>

    <h1 class="error-screen__heading">{heading}</h1>

    <p class="error-screen__message" role="alert">
      {message}
    </p>

    <a href={buttonHref} class="error-screen__btn">
      {buttonLabel}
    </a>
  </div>
</AppShell>

<style>
  .error-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
    padding: 3rem 1rem;
    max-width: 40rem;
    margin: 0 auto;
    text-align: center;
  }

  .error-screen__icon {
    font-size: 4rem;
    line-height: 1;
  }

  .error-screen__heading {
    font-size: 1.875rem;
    font-weight: 700;
    color: #111827;
    margin: 0;
  }

  .error-screen__message {
    font-size: 1rem;
    color: #374151;
    margin: 0;
  }

  .error-screen__btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    background-color: var(--brand-primary, #2563eb);
    color: #fff;
    border-radius: 0.375rem;
    font-size: 1rem;
    font-weight: 500;
    text-decoration: none;
  }

  .error-screen__btn:hover {
    opacity: 0.9;
  }
</style>
