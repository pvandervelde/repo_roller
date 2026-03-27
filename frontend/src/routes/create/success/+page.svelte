<script lang="ts">
  import type { PageData } from './$types';

  const { data }: { data: PageData } = $props();
  const appName = $derived(data.brandConfig.appName);
</script>

<svelte:head>
  <title>Repository created — {appName}</title>
</svelte:head>

<main>
  <h1>Repository created!</h1>
  <p>Success screen implementation coming in task 13.12.</p>
</main>
