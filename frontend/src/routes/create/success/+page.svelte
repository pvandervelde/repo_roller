<script lang="ts">
  import type { PageData } from './$types';
  import AppShell from '$lib/components/AppShell.svelte';

  const { data }: { data: PageData } = $props();

  const repoFullName = $derived(data.repo ?? '');
  const isValid = $derived(repoFullName.includes('/') && repoFullName.length > 2);
  const githubUrl = $derived(isValid ? `https://github.com/${repoFullName}` : '');
</script>

<svelte:head>
  <title>Repository created — {data.brandConfig.appName}</title>
</svelte:head>

<AppShell
  appName={data.brandConfig.appName}
  logoUrl={data.brandConfig.logoUrl}
  logoUrlDark={data.brandConfig.logoUrlDark}
  logoAlt={data.brandConfig.logoAlt}
  userLogin={data.session?.userLogin ?? ''}
  userAvatarUrl={data.session?.userAvatarUrl ?? null}
>
  <div class="success">
    {#if isValid}
      <span class="success__icon" aria-hidden="true">✓</span>

      <h1 class="success__heading">Repository created!</h1>

      <p class="success__repo-name">
        <code>{repoFullName}</code>
      </p>

      <a
        href={githubUrl}
        target="_blank"
        rel="noopener noreferrer"
        class="success__repo-link"
        aria-label="View {repoFullName} on GitHub (opens in new tab)"
      >
        {githubUrl}
      </a>

      <a href={githubUrl} target="_blank" rel="noopener noreferrer" class="success__btn-primary">
        View repository on GitHub ↗
      </a>
    {:else}
      <p class="success__invalid-msg" role="status">
        Your repository was created. Check your GitHub organization to find it.
      </p>
    {/if}

    <a href="/create" class="success__btn-secondary">Create another repository</a>
  </div>
</AppShell>

<style>
  .success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
    padding: 3rem 1rem;
    max-width: 40rem;
    margin: 0 auto;
    text-align: center;
  }

  .success__icon {
    font-size: 4rem;
    color: #16a34a;
    line-height: 1;
  }

  .success__heading {
    font-size: 1.875rem;
    font-weight: 700;
    color: #111827;
    margin: 0;
  }

  .success__repo-name {
    font-size: 1.125rem;
    color: #374151;
    margin: 0;
  }

  .success__repo-name code {
    font-family: ui-monospace, 'Cascadia Code', 'Source Code Pro', Menlo, Consolas, monospace;
    background-color: #f3f4f6;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
  }

  .success__repo-link {
    font-size: 0.9375rem;
    color: var(--brand-primary, #2563eb);
    text-decoration: underline;
    word-break: break-all;
  }

  .success__btn-primary {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    background-color: var(--brand-primary, #2563eb);
    color: #fff;
    border-radius: 0.375rem;
    font-size: 1rem;
    font-weight: 500;
    text-decoration: none;
  }

  .success__btn-primary:hover {
    opacity: 0.9;
  }

  .success__btn-secondary {
    display: inline-block;
    padding: 0.625rem 1.25rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    color: #374151;
    text-decoration: none;
  }

  .success__btn-secondary:hover {
    background-color: #f9fafb;
  }

  .success__invalid-msg {
    font-size: 1rem;
    color: #374151;
    margin: 0;
  }
</style>
