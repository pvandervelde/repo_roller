<script lang="ts">
  import type { PageData } from './$types';
  import { onMount, tick } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import StepProgress from '$lib/components/StepProgress.svelte';
  import TemplateGrid from '$lib/components/TemplateGrid.svelte';
  import RepositoryNameField from '$lib/components/RepositoryNameField.svelte';
  import RepositoryTypePicker from '$lib/components/RepositoryTypePicker.svelte';
  import RepositorySummary from '$lib/components/RepositorySummary.svelte';
  import VariableField from '$lib/components/VariableField.svelte';
  import InlineAlert from '$lib/components/InlineAlert.svelte';
  import { goto } from '$app/navigation';
  import {
    listTemplates,
    getTemplateDetails,
    listRepositoryTypes,
    createRepository,
  } from '$lib/api/client';
  import type { GetTemplateDetailsResponse, TemplateSummary } from '$lib/api/types';
  import type { NameValidationResult } from '$lib/components/RepositoryNameField.svelte';
  import type { RepositoryTypeOption } from '$lib/components/RepositoryTypePicker.svelte';

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

  // Step 2 state
  let repoName = $state('');
  let nameValidation = $state<NameValidationResult>({ status: 'idle' });
  let selectedTypeName: string | null = $state(null);
  let selectedTeamName: string | null = $state(null);
  let visibility = $state<'private' | 'public'>('private');
  let availableTypes: RepositoryTypeOption[] = $state([]);
  let teamsLoading = $state(false);
  let teamsError = $state(false);
  let teams: Array<{ slug: string; name: string }> = $state([]);

  // ---------------------------------------------------------------------------
  // Derived values
  // ---------------------------------------------------------------------------

  const hasVariables = $derived((templateDetails?.variables.length ?? 0) > 0);

  const steps = $derived(
    hasVariables ? ['Choose template', 'Settings', 'Variables'] : ['Choose template', 'Settings'],
  );

  const step1NextEnabled = $derived(
    selectedTemplateName !== null &&
      templateDetails !== null &&
      !templateDetailsLoading &&
      templateDetailsError === null,
  );

  // Step 2's "next/create" is enabled when name validation is available or check_failed
  // UX-ASSERT-011 (disabled while checking), UX-ASSERT-012 (disabled when taken),
  // UX-ASSERT-028 (enabled when check_failed)
  const step2Actionable = $derived(
    repoName.length > 0 &&
      (nameValidation.status === 'available' || nameValidation.status === 'check_failed'),
  );

  const templateTypePolicy = $derived.by<'fixed' | 'preferable' | 'optional'>(() => {
    const p = templateDetails?.repository_type?.policy;
    if (p === 'fixed' || p === 'preferable' || p === 'optional') return p;
    return 'optional';
  });

  const templateTypeName = $derived(templateDetails?.repository_type?.type_name ?? null);

  // Sorted variable list: required first, then optional (UX spec)
  const sortedVariables = $derived(
    [...(templateDetails?.variables ?? [])].sort((a, b) => {
      if (a.required === b.required) return 0;
      return a.required ? -1 : 1;
    }),
  );

  // Step 3 Create enabled when all required variables have non-empty values (UX-ASSERT-016)
  const step3CreateEnabled = $derived(
    (templateDetails?.variables ?? []).every(
      (v) => !v.required || (variableValues[v.name] ?? '').trim().length > 0,
    ),
  );

  // Creation error for the no-variables (direct create) path
  let createError: string | null = $state(null);
  let typesLoadedForStep2 = $state(false);

  // Step 3 state — variable values keyed by variable name
  let variableValues = $state<Record<string, string>>({});

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------

  /** Convert snake_case / kebab-case to Title Case for variable labels. */
  function toTitleCase(name: string): string {
    return name.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

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
    if (currentStep === 1) {
      // Entering step 2 for the first time: load types
      if (!typesLoadedForStep2) {
        typesLoadedForStep2 = true;
        loadRepositoryTypes();
      }
    }
    currentStep += 1;
    await tick();
    if (currentStep === 2) step2Heading?.focus();
    else if (currentStep === 3) step3Heading?.focus();
  }

  async function loadRepositoryTypes() {
    try {
      const names = await listRepositoryTypes(data.organization);
      availableTypes = names.map((n) => ({ name: n, description: '' }));
    } catch {
      // Leave availableTypes empty — picker degrades gracefully
    }
  }

  async function handleStep2Action() {
    if (hasVariables) {
      // Initialise variable values from defaults before advancing (UX-ASSERT-017)
      const initial: Record<string, string> = {};
      for (const v of templateDetails?.variables ?? []) {
        initial[v.name] = variableValues[v.name] ?? v.default_value ?? '';
      }
      variableValues = initial;
      await advanceStep();
    } else {
      await submitCreate();
    }
  }

  async function submitCreate(variables: Record<string, string> = {}) {
    createError = null;
    try {
      await createRepository({
        organization: data.organization,
        name: repoName,
        template: selectedTemplateName!,
        visibility,
        team: selectedTeamName ?? undefined,
        repository_type: selectedTypeName ?? undefined,
        variables: Object.keys(variables).length > 0 ? variables : undefined,
      });
      await goto('/create/success');
    } catch {
      createError = 'Failed to create repository. Please try again.';
    }
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
          disabled={!step1NextEnabled}
          onclick={advanceStep}
        >
          Next: Repository settings →
        </button>
      </div>
    {:else if currentStep === 2}
      <h1 tabindex="-1" bind:this={step2Heading} class="wizard__heading">Repository settings</h1>

      {#if createError}
        <InlineAlert variant="error" message={createError} />
      {/if}

      <div class="wizard__step2-fields">
        <RepositoryNameField
          value={repoName}
          organization={data.organization}
          onchange={(v) => {
            repoName = v;
          }}
          onvalidationResult={(r) => {
            nameValidation = r;
          }}
        />

        <RepositoryTypePicker
          policy={templateTypePolicy}
          {templateTypeName}
          {availableTypes}
          {selectedTypeName}
          onchange={(v) => {
            selectedTypeName = v;
          }}
        />

        <div class="wizard__field">
          <label for="team-select" class="wizard__label">Team (optional)</label>
          {#if teamsLoading}
            <select id="team-select" disabled class="wizard__select">
              <option>Loading teams…</option>
            </select>
          {:else if teamsError}
            <InlineAlert variant="info" message="Team configuration unavailable." />
          {:else}
            <select
              id="team-select"
              class="wizard__select"
              onchange={(e) => {
                selectedTeamName = (e.target as HTMLSelectElement).value || null;
              }}
            >
              <option value="">No specific team</option>
              {#each teams as team}
                <option value={team.slug}>{team.name}</option>
              {/each}
            </select>
          {/if}
          {#if !teamsError}
            <p class="wizard__helper">
              Select your team to apply team-specific configuration defaults.
            </p>
          {/if}
        </div>

        {#if !hasVariables}
          <RepositorySummary
            organization={data.organization}
            repositoryName={repoName}
            templateName={selectedTemplateName ?? ''}
            typeName={selectedTypeName}
            {visibility}
            teamName={selectedTeamName}
          />
        {/if}
      </div>

      <div class="wizard__actions">
        <button type="button" class="wizard__btn-back" onclick={retreatStep}>← Back</button>
        <button
          type="button"
          class="wizard__btn-next"
          disabled={!step2Actionable}
          onclick={handleStep2Action}
        >
          {hasVariables ? 'Next: Variables →' : 'Create Repository'}
        </button>
      </div>
    {:else if currentStep === 3}
      <h1 tabindex="-1" bind:this={step3Heading} class="wizard__heading">Template variables</h1>

      {#if createError}
        <InlineAlert variant="error" message={createError} />
      {/if}

      <div class="wizard__step2-fields">
        {#each sortedVariables as variable (variable.name)}
          <VariableField
            variableName={variable.name}
            label={toTitleCase(variable.name)}
            description={variable.description ?? null}
            required={variable.required}
            defaultValue={variable.default_value ?? null}
            value={variableValues[variable.name] ?? ''}
            onchange={(v) => {
              variableValues = { ...variableValues, [variable.name]: v };
            }}
          />
        {/each}

        <RepositorySummary
          organization={data.organization}
          repositoryName={repoName}
          templateName={selectedTemplateName ?? ''}
          typeName={selectedTypeName}
          {visibility}
          teamName={selectedTeamName}
        />
      </div>

      <div class="wizard__actions">
        <button type="button" class="wizard__btn-back" onclick={retreatStep}>← Back</button>
        <button
          type="button"
          class="wizard__btn-create"
          disabled={!step3CreateEnabled}
          onclick={() => submitCreate(variableValues)}
        >
          Create Repository
        </button>
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

  .wizard__step2-fields {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .wizard__field {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .wizard__label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .wizard__select {
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    background-color: #fff;
    color: #111827;
  }

  .wizard__select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .wizard__helper {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }
</style>
