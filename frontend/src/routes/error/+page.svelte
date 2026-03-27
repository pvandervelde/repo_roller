<script lang="ts">
  import type { PageData } from './$types';

  const { data }: { data: PageData } = $props();
  const appName = $derived(data.brandConfig.appName);
</script>

<svelte:head>
  <title>Error — {appName}</title>
</svelte:head>

<main>
  <h1>Something went wrong</h1>
  <p>An unexpected error occurred. If this keeps happening, contact your platform team.</p>
  <a href="/create" role="button">Try again</a>
</main>
