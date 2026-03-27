<script lang="ts">
  import type { PageData } from './$types';

  const { data }: { data: PageData } = $props();
  const appName = data.brandConfig.appName;
</script>

<svelte:head>
  <title>Access denied — {appName}</title>
</svelte:head>

<main>
  <h1>Sign-in could not be completed</h1>
  <p>There was a problem connecting to GitHub. This is usually temporary.</p>
  <a href="/sign-in" role="button">Try again</a>
</main>
