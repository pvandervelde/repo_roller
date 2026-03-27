<script lang="ts">
  import type { PageData } from './$types';

  const { data }: { data: PageData } = $props();
  const appName = data.brandConfig.appName;
</script>

<svelte:head>
  <title>Sign in — {appName}</title>
</svelte:head>

<main>
  <h1>Sign in to {appName}</h1>
  <p>Create standardized GitHub repositories from your organization's templates.</p>
  <a href="/auth/github" role="button">Sign in with GitHub</a>
</main>
