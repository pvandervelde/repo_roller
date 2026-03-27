<script lang="ts">
  import type { PageData } from './$types';

  const { data }: { data: PageData } = $props();
  const appName = data.brandConfig.appName;
</script>

<svelte:head>
  <title>Create repository — {appName}</title>
</svelte:head>

<main>
  <h1>Create repository</h1>
  <p>Wizard implementation coming in tasks 13.7–13.11.</p>
</main>
