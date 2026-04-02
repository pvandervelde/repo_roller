<script lang="ts">
  import type { PageData } from './$types';
  import { onMount, tick } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import StepProgress from '$lib/components/StepProgress.svelte';
  import TemplateGrid from '$lib/components/TemplateGrid.svelte';
  import { listTemplates, getTemplateDetails } from '$lib/api/client';
  import type { GetTemplateDetailsResponse, TemplateSummary } from '$lib/api/types';

  const { data }: { data: PageData } = $props();

  // ---------------------------------------------------------------------------
  // Wizard state
  // ---------------------------------------------------------------------------

  let currentStep = $state(1);
  let templates: TemplateSummary[] = $state([]);
  let templatesLoading = $state(true);
  let templatesError: string | null = $state(null);
  let selectedTemplateName: string | null = $state(null);
  let templateDetails = $state<GetTemplateDetailsResponse | null>(null);
  let templateDetailsLoading = $state(false);
  let templateDetailsError: string | null = $state(null);
  let repoName = $state('');

  // ---------------------------------------------------------------------------
  // Derived values
  // ---------------------------------------------------------------------------

  const hasVariables = $derived((templateDetails?.variables.length ?? 0) > 0);

  const steps = $derived(
    hasVariables ? ['Choose template', 'Settings', 'Variables'] : ['Choose template', 'Settings'],
  );

  const nextEnabled = $derived(
    currentStep === 1 &&
      selectedTemplateName !== null &&
      templateDetails !== null &&
      !templateDetailsLoading &&
      templateDetailsError === null,
  );

  // ---------------------------------------------------------------------------
  // DOM refs for focus management  (UX-ASSERT-024)
  // ---------------------------------------------------------------------------

  let step1Heading: HTMLElement | null = $state(null);
  let step2Heading: HTMLElement | null = $state(null);
  let step3Heading: HTMLElement | null = $state(null);

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(() => {
    loadTemplates();

    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (selectedTemplateName !== null || repoName.length > 0) {
        e.preventDefault();
      }
    };
    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  });

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  async function loadTemplates() {
    templatesLoading = true;
    templatesError = null;
    try {
      templates = await listTemplates(data.organization);
    } catch {
      templatesError = 'Could not load templates.';
    } finally {
      templatesLoading = false;
    }
  }

  async function handleTemplateSelect(name: string) {
    selectedTemplateName = name;
    templateDetails = null;
    templateDetailsLoading = true;
    templateDetailsError = null;
    try {
      templateDetails = await getTemplateDetails(data.organization, name);
    } catch {
      templateDetailsError = 'Could not load template details.';
    } finally {
      templateDetailsLoading = false;
    }
  }

  async function advanceStep() {
    currentStep += 1;
    await tick();
    if (currentStep === 2) step2Heading?.focus();
    else if (currentStep === 3) step3Heading?.focus();
  }

  async function retreatStep() {
    currentStep -= 1;
    await tick();
    if (currentStep === 1) step1Heading?.focus();
    else if (currentStep === 2) step2Heading?.focus();
  }
</script>

<svelte:head>
  <title>Create repository — {data.brandConfig.appName}</title>
</svelte:head>

<AppShell
  appName={data.brandConfig.appName}
  logoUrl={data.brandConfig.logoUrl}
  logoUrlDark={data.brandConfig.logoUrlDark}
  logoAlt={data.brandConfig.logoAlt}
  userLogin={data.session?.userLogin ?? ''}
  userAvatarUrl={data.session?.userAvatarUrl ?? null}
>
  <div class="wizard">
    <StepProgress {steps} {currentStep} />

    {#if currentStep === 1}
      <h1 tabindex="-1" bind:this={step1Heading} class="wizard__heading">Choose a template</h1>

      <TemplateGrid
        {templates}
        {selectedTemplateName}
        loading={templatesLoading}
        error={templatesError}
        ontemplateSelect={handleTemplateSelect}
        onretry={loadTemplates}
      />

      {#if templateDetailsError}
        <p class="wizard__detail-error" role="alert">{templateDetailsError}</p>
      {/if}

      <div class="wizard__actions wizard__actions--step1">
        <button
          type="button"
          class="wizard__btn-next"
          disabled={!nextEnabled}
          onclick={advanceStep}
        >
          Next: Repository settings →
        </button>
      </div>
    {:else if currentStep === 2}
      <h1 tabindex="-1" bind:this={step2Heading} class="wizard__heading">Repository settings</h1>

      <p class="wizard__placeholder">Step 2 implementation coming in task 13.9.</p>

      <div class="wizard__actions">
        <button type="button" class="wizard__btn-back" onclick={retreatStep}>← Back</button>
        <button type="button" class="wizard__btn-next" disabled>
          {hasVariables ? 'Next: Variables →' : 'Create Repository'}
        </button>
      </div>
    {:else if currentStep === 3}
      <h1 tabindex="-1" bind:this={step3Heading} class="wizard__heading">Template variables</h1>

      <p class="wizard__placeholder">Step 3 implementation coming in task 13.10.</p>

      <div class="wizard__actions">
        <button type="button" class="wizard__btn-back" onclick={retreatStep}>← Back</button>
        <button type="button" class="wizard__btn-create" disabled> Create Repository </button>
      </div>
    {/if}
  </div>
</AppShell>

<style>
  .wizard {
    max-width: 56rem;
    margin: 0 auto;
    padding: 2rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .wizard__heading {
    font-size: 1.5rem;
    font-weight: 700;
    color: #111827;
    margin: 0;
  }

  .wizard__heading:focus {
    outline: none;
  }

  .wizard__actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 1rem;
  }

  .wizard__actions--step1 {
    justify-content: flex-end;
  }

  .wizard__btn-next,
  .wizard__btn-create {
    padding: 0.625rem 1.25rem;
    background-color: var(--brand-primary, #2563eb);
    color: #fff;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .wizard__btn-next:disabled,
  .wizard__btn-create:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .wizard__btn-back {
    padding: 0.625rem 1rem;
    background: none;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    color: #374151;
    cursor: pointer;
  }

  .wizard__btn-back:hover {
    background-color: #f9fafb;
  }

  .wizard__detail-error {
    font-size: 0.875rem;
    color: #b91c1c;
  }

  .wizard__placeholder {
    font-size: 0.875rem;
    color: #6b7280;
  }
</style>
