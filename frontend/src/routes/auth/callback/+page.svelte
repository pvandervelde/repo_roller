<script lang="ts">
  import type { PageData } from './$types';

  const { data }: { data: PageData } = $props();
  const appName = $derived(data.brandConfig.appName);
</script>

<svelte:head>
  <title>Signing in… — {appName}</title>
</svelte:head>

<main>
  <h1>Completing sign-in</h1>
  <p>You'll be redirected in a moment.</p>
</main>
