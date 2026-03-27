<script lang="ts">
  import '../app.css';
  import type { LayoutData } from './$types';
  import type { Snippet } from 'svelte';
  import { buildBrandCssBlock } from '$lib/brand';

  const { data, children }: { data: LayoutData; children: Snippet } = $props();

  const brandCss = buildBrandCssBlock(data.brandConfig);
</script>

<svelte:head>
  {#if brandCss}
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html `<style>${brandCss}</style>`}
  {/if}
</svelte:head>

{@render children()}
