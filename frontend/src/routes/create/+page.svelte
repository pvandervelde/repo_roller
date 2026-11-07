<script lang="ts">
  import type { PageData } from './$types';
  import { onMount, tick } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import StepProgress from '$lib/components/StepProgress.svelte';
  import TemplateGrid from '$lib/components/TemplateGrid.svelte';
  import TemplateDetailsPanel from '$lib/components/TemplateDetailsPanel.svelte';
  import RepositoryNameField from '$lib/components/RepositoryNameField.svelte';
  import RepositoryTypePicker from '$lib/components/RepositoryTypePicker.svelte';
  import RepositorySummary from '$lib/components/RepositorySummary.svelte';
  import VariableField from '$lib/components/VariableField.svelte';
  import InlineAlert from '$lib/components/InlineAlert.svelte';
  import CreationOverlay from '$lib/components/CreationOverlay.svelte';
  import { goto } from '$app/navigation';
  import { enhance } from '$app/forms';
  import {
    listTemplates,
    getTemplateDetails,
    listRepositoryTypes,
    listTeams,
    createRepository,
    getCreationCapabilities,
  } from '$lib/api/client';
  import {
    ApiConflictError,
    ApiAuthError,
    ApiNetworkError,
    ApiValidationError,
  } from '$lib/api/errors';
  import { withSpan, SPAN_NAMES } from '$lib/observability/spans';
  import type {
    CreateRepositoryRequest,
    CreationCapabilitiesResponse,
    GetTemplateDetailsResponse,
    TemplateSummary,
  } from '$lib/api/types';
  import type { NameValidationResult } from '$lib/components/RepositoryNameField.svelte';
  import type { RepositoryTypeOption } from '$lib/components/RepositoryTypePicker.svelte';

  const { data }: { data: PageData } = $props();

  // DOM reference for the hidden sign-out form (used by handleSignOut)
  let signOutForm: HTMLFormElement | undefined;

  function handleSignOut() {
    signOutForm?.requestSubmit();
  }

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
  // Fork/Mirror creation-mode state (WUI-005, fork-mirror-operations.md §8a/§8b)
  // ---------------------------------------------------------------------------

  const DISABLED_CAPABILITIES: CreationCapabilitiesResponse = {
    forkEnabled: false,
    sourceHistoryMirrorEnabled: false,
    sourceOnlyMirrorEnabled: false,
    allowedSourceOwners: [],
    forkTemplate: null,
    sourceHistoryMirrorTemplate: null,
    sourceOnlyMirrorTemplate: null,
  };

  // Resolved from GET /orgs/:org/creation-capabilities. Any discovery failure
  // other than AmbiguousTemplateDesignation degrades silently by leaving this
  // at the all-disabled default — the pre-existing template-only flow is
  // never affected by a failed/erroring discovery call.
  let capabilities = $state<CreationCapabilitiesResponse>(DISABLED_CAPABILITIES);
  // Distinct, visible message for the one discovery failure that must NOT be
  // degraded silently (AmbiguousTemplateDesignation).
  let capabilitiesAlert: string | null = $state(null);

  let creationModeChoice = $state<'template' | 'fork' | 'mirror'>('template');
  let sourceReference = $state('');
  // The user's explicit history-preservation pick; reset to null (falls back
  // to the implicit/default choice) whenever the creation mode changes.
  let historyChoiceExplicit = $state<'preserve' | 'source-only' | null>(null);

  // ---------------------------------------------------------------------------
  // Derived values
  // ---------------------------------------------------------------------------

  const showModeChoice = $derived(
    capabilities.forkEnabled ||
      capabilities.sourceHistoryMirrorEnabled ||
      capabilities.sourceOnlyMirrorEnabled,
  );

  const mirrorHistoryToggleVisible = $derived(
    capabilities.sourceHistoryMirrorEnabled && capabilities.sourceOnlyMirrorEnabled,
  );

  // Implicit choice when only one mirror sub-type is enabled; the user's
  // default when both are enabled (WUI-005 judgment call #4).
  const defaultHistoryChoice = $derived<'preserve' | 'source-only'>(
    capabilities.sourceHistoryMirrorEnabled ? 'preserve' : 'source-only',
  );

  const effectiveHistoryChoice = $derived(historyChoiceExplicit ?? defaultHistoryChoice);
  const preserveHistory = $derived(effectiveHistoryChoice === 'preserve');

  // Basic client-side shape check: exactly two non-empty segments separated
  // by exactly one '/'.
  const parsedSource = $derived.by<{ owner: string; name: string } | null>(() => {
    if (creationModeChoice === 'template') return null;
    const match = /^([^/]+)\/([^/]+)$/.exec(sourceReference.trim());
    return match ? { owner: match[1], name: match[2] } : null;
  });

  const sourceReferenceError = $derived.by<string | null>(() => {
    if (creationModeChoice === 'template') return null;
    if (sourceReference.trim().length === 0) return null;
    if (parsedSource !== null) return null;
    return 'Enter the source repository as "owner/repo".';
  });

  const sourceReferenceValid = $derived(creationModeChoice !== 'template' && parsedSource !== null);

  // Advisory only — the allow-list is non-additive per team/repositoryType
  // scope and the server remains authoritative; never blocks submission.
  const sourceOwnerWarning = $derived.by<string | null>(() => {
    if (creationModeChoice === 'template' || parsedSource === null) return null;
    if (capabilities.allowedSourceOwners.length === 0) return null;
    if (capabilities.allowedSourceOwners.includes(parsedSource.owner)) return null;
    return `'${parsedSource.owner}' is not in the list of commonly used source owners for this organization.`;
  });

  // The designated settings template (if any) for the currently-selected
  // Fork/Mirror mode — informational-only, never user-selectable (WUI-005).
  const designatedTemplateName = $derived.by<string | null>(() => {
    if (creationModeChoice === 'fork') return capabilities.forkTemplate;
    if (creationModeChoice === 'mirror') {
      return preserveHistory
        ? capabilities.sourceHistoryMirrorTemplate
        : capabilities.sourceOnlyMirrorTemplate;
    }
    return null;
  });

  // Label shown in RepositorySummary's "Template" chip for all three modes.
  const summaryTemplateLabel = $derived.by<string>(() => {
    if (creationModeChoice === 'fork') return `Fork of ${sourceReference}`;
    if (creationModeChoice === 'mirror') return `Mirror of ${sourceReference}`;
    return selectedTemplateName ?? '';
  });

  function selectCreationMode(mode: 'template' | 'fork' | 'mirror') {
    if (creationModeChoice === mode) return;
    creationModeChoice = mode;
    // Adversarial-robustness: never leak Fork/Mirror form state across a
    // round-trip through another mode (see "rapid mode switching" tests).
    sourceReference = '';
    historyChoiceExplicit = null;
  }

  // Fork/Mirror never produce a Variables step — the designated settings
  // template (if any) is informational-only on Review, never user-editable.
  const hasVariables = $derived(
    creationModeChoice === 'template' && (templateDetails?.variables.length ?? 0) > 0,
  );

  // A final "Review" step is always appended; the Create action lives there
  // exclusively (never on Settings/Variables, which only ever advance).
  const steps = $derived(
    hasVariables
      ? ['Choose template', 'Settings', 'Variables', 'Review']
      : ['Choose template', 'Settings', 'Review'],
  );

  // Step numbers are derived rather than hardcoded so the Review step's
  // position correctly follows Variables when present, or Settings otherwise.
  const variablesStepNumber = $derived(hasVariables ? 3 : null);
  const reviewStepNumber = $derived(steps.length);

  const step1NextEnabled = $derived(
    creationModeChoice === 'template'
      ? selectedTemplateName !== null &&
          templateDetails !== null &&
          !templateDetailsLoading &&
          templateDetailsError === null
      : sourceReferenceValid,
  );

  // Step 2's "next" is enabled when name validation is available or check_failed
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

  // Step 3's "Next: Review" enabled when all required variables have non-empty
  // values (UX-ASSERT-016, generalized to gate advancement rather than create)
  const step3NextEnabled = $derived(
    (templateDetails?.variables ?? []).every(
      (v) => !v.required || (variableValues[v.name] ?? '').trim().length > 0,
    ),
  );

  // Every configured variable, with its resolved value, for display on Review
  // (previously never surfaced anywhere in the wizard).
  const reviewVariables = $derived(
    (templateDetails?.variables ?? []).map((v) => ({
      name: v.name,
      value: variableValues[v.name] ?? v.default ?? '',
    })),
  );

  // ---------------------------------------------------------------------------
  // Disabled-button tooltip reasons
  // ---------------------------------------------------------------------------

  const step1NextTooltip = $derived.by<string | null>(() => {
    if (step1NextEnabled) return null;
    if (creationModeChoice !== 'template') {
      return 'Enter a valid source repository ("owner/repo") to continue';
    }
    if (!selectedTemplateName) return 'Select a template to continue';
    if (templateDetailsLoading) return 'Loading template details…';
    if (templateDetailsError) return 'Template details could not be loaded — try again';
    return null;
  });

  const step2ActionTooltip = $derived.by<string | null>(() => {
    if (step2Actionable) return null;
    if (!repoName) return 'Enter a repository name to continue';
    const s = nameValidation.status;
    if (s === 'checking') return 'Checking name availability…';
    if (s === 'taken') return 'This repository name is already taken — choose another';
    if (s === 'invalid_format') return 'Fix the repository name format to continue';
    return 'Enter a valid, available repository name to continue';
  });

  const step3NextTooltip = $derived.by<string | null>(() => {
    if (step3NextEnabled) return null;
    return 'Fill in all required template variables to continue';
  });

  // Creation error, surfaced on whichever step submitCreate routes back to
  let createError: string | null = $state(null);
  let creatingRepo = $state(false);
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
  // DOM ref for focus management (UX-ASSERT-024) — a single ref is sufficient
  // because exactly one step heading is ever mounted at a time.
  // ---------------------------------------------------------------------------

  let wizardHeading: HTMLElement | null = $state(null);

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(() => {
    loadTemplates();
    loadCapabilities();

    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      // Guard disabled while overlay is active (creation is in flight)
      if (creatingRepo) return;
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
      templates = await withSpan(SPAN_NAMES.LOAD_TEMPLATES, () => listTemplates(data.organization));
    } catch {
      templatesError = 'Could not load templates.';
    } finally {
      templatesLoading = false;
    }
  }

  // WUI-005: discover Fork/Mirror capability state. Any failure other than
  // AmbiguousTemplateDesignation degrades silently (capabilities stays at
  // its all-disabled default, defined above) — the template-only flow must
  // never be affected by a broken/unavailable discovery endpoint.
  async function loadCapabilities() {
    try {
      capabilities = await getCreationCapabilities(data.organization, {});
    } catch (err) {
      if (
        err instanceof ApiConflictError &&
        err.errorDetails.code === 'AmbiguousTemplateDesignation'
      ) {
        capabilitiesAlert = err.errorDetails.message;
      }
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

  function handleTemplateDeselect() {
    selectedTemplateName = null;
    templateDetails = null;
    templateDetailsError = null;
  }

  async function advanceStep() {
    if (currentStep === 1) {
      // Entering step 2 for the first time: load types and teams
      if (!typesLoadedForStep2) {
        typesLoadedForStep2 = true;
        loadRepositoryTypes();
        loadTeams();
      }
    }
    currentStep += 1;
    await tick();
    wizardHeading?.focus();
  }

  async function loadRepositoryTypes() {
    try {
      availableTypes = await listRepositoryTypes(data.organization);
    } catch {
      // Leave availableTypes empty — picker degrades gracefully
    }
  }

  async function loadTeams() {
    teamsLoading = true;
    teamsError = false;
    try {
      teams = await listTeams(data.organization);
    } catch {
      teamsError = true;
    } finally {
      teamsLoading = false;
    }
  }

  // Settings step's action always advances — to Variables (if the template
  // has any) or straight to Review otherwise. The Create action lives on
  // Review exclusively.
  async function handleStep2Action() {
    if (hasVariables) {
      // Initialise variable values from defaults before advancing (UX-ASSERT-017)
      const initial: Record<string, string> = {};
      for (const v of templateDetails?.variables ?? []) {
        initial[v.name] = variableValues[v.name] ?? v.default ?? '';
      }
      variableValues = initial;
    }
    await advanceStep();
  }

  /**
   * Builds the POST /api/v1/repositories request body for the current
   * creation mode. Fork/Mirror never send `template`/`variables` (the
   * backend auto-discovers the designated template, if any) and standard
   * template-based creation never sends `creationMode` (fork-mirror-
   * operations.md §8a).
   */
  function buildCreateRequest(variables: Record<string, string>): CreateRepositoryRequest {
    const base = {
      organization: data.organization,
      name: repoName,
      visibility,
      team: selectedTeamName ?? undefined,
      repositoryType: selectedTypeName ?? undefined,
    };

    if (creationModeChoice === 'fork' && parsedSource !== null) {
      return { ...base, creationMode: { type: 'fork', source: parsedSource } };
    }
    if (creationModeChoice === 'mirror' && parsedSource !== null) {
      return {
        ...base,
        creationMode: { type: 'mirror', source: parsedSource, preserveHistory },
      };
    }
    return {
      ...base,
      template: selectedTemplateName!,
      variables: Object.keys(variables).length > 0 ? variables : undefined,
    };
  }

  async function submitCreate(variables: Record<string, string> = {}) {
    // Guards against duplicate submissions from rapid repeat clicks on the
    // Review step's "Create Repository" button (UX-ASSERT-018, security).
    if (creatingRepo) return;

    createError = null;
    creatingRepo = true;
    try {
      const result = await withSpan(SPAN_NAMES.CREATE_REPOSITORY_SUBMIT, () =>
        createRepository(buildCreateRequest(variables)),
      );
      // Success: navigate away (guard already suppressed while creatingRepo=true)
      await goto(`/create/success?repo=${encodeURIComponent(result.repository.fullName)}`);
    } catch (err) {
      creatingRepo = false;

      if (err instanceof ApiConflictError) {
        const code = err.errorDetails.code ?? '';
        if (code === 'TemplateNotFound' || err.errorDetails.message?.includes('template')) {
          // UX-ASSERT-020: template not found — back to step 1, refresh list
          currentStep = 1;
          await tick();
          wizardHeading?.focus();
          await loadTemplates();
          createError = `The template '${selectedTemplateName}' is no longer available. Please choose a different template.`;
          selectedTemplateName = null;
          templateDetails = null;
        } else {
          // UX-ASSERT-019: name already taken (race condition) — back to step 2
          currentStep = 2;
          await tick();
          wizardHeading?.focus();
          nameValidation = { status: 'taken', name: repoName };
          createError = `A repository named '${repoName}' was created while you were filling in this form. Please choose a different name.`;
        }
      } else if (err instanceof ApiAuthError) {
        // WUIE-003: Fork/Mirror-specific 403s get a distinct, specific
        // message instead of the generic permission-denied fallback. Form
        // state is preserved — currentStep is left unchanged (Review).
        const code = err.errorDetails.code ?? '';
        if (code === 'CapabilityDisabled') {
          createError = `This creation mode is disabled for this organization. ${err.errorDetails.message}`;
        } else if (code === 'SourceOwnerNotAllowed') {
          createError = err.errorDetails.message;
        } else {
          // 403 permission denied — show inline on current step
          createError =
            "You don't have permission to create repositories in this organization. Contact your administrator.";
        }
      } else if (err instanceof ApiValidationError) {
        // WUIE-003: TemplateNotDesignated (400) gets a distinct, specific
        // message instead of falling through to the generic 5xx fallback.
        const code = err.errorDetails.code ?? '';
        if (code === 'TemplateNotDesignated') {
          createError = `No settings template is designated for this creation mode. ${err.errorDetails.message}`;
        } else {
          createError = err.errorDetails.message;
        }
      } else if (err instanceof ApiNetworkError) {
        createError = 'Could not reach the server. Check your connection and try again.';
      } else {
        // UX-ASSERT-021: 5xx server error
        createError =
          'GitHub is temporarily unavailable. Your repository was not created. Please try again.';
      }
    }
  }

  async function retreatStep() {
    // Generic decrement works for every step: Review always sits immediately
    // after Variables (when present) or Settings (when not), so stepping
    // back by one always lands on the correct prior step.
    currentStep -= 1;
    await tick();
    wizardHeading?.focus();
  }
</script>

<svelte:head>
  <title>Create repository — {data.brandConfig.appName}</title>
</svelte:head>

<form
  bind:this={signOutForm}
  method="POST"
  action="?/signOut"
  aria-hidden="true"
  style="display: none;"
  use:enhance
>
  <button type="submit" tabindex="-1">Sign out</button>
</form>

<AppShell
  appName={data.brandConfig.appName}
  logoUrl={data.brandConfig.logoUrl}
  logoUrlDark={data.brandConfig.logoUrlDark}
  logoAlt={data.brandConfig.logoAlt}
  userLogin={data.session?.userLogin ?? ''}
  userAvatarUrl={data.session?.userAvatarUrl ?? null}
  onsignOut={handleSignOut}
>
  <div
    class="wizard"
    class:wizard--step1={currentStep === 1}
    aria-hidden={creatingRepo ? 'true' : undefined}
  >
    <StepProgress {steps} {currentStep} />

    {#if currentStep === 1}
      <h1 tabindex="-1" bind:this={wizardHeading} class="wizard__heading">Choose a template</h1>

      {#if capabilitiesAlert}
        <InlineAlert variant="error" message={capabilitiesAlert} />
      {/if}
      {#if createError}
        <InlineAlert variant="error" message={createError} />
      {/if}

      {#if showModeChoice}
        <div role="radiogroup" aria-label="Creation mode" class="wizard__mode-choice">
          <label class="wizard__mode-option">
            <input
              type="radio"
              name="creation-mode"
              value="template"
              checked={creationModeChoice === 'template'}
              onchange={() => selectCreationMode('template')}
            />
            Template
          </label>
          {#if capabilities.forkEnabled}
            <label class="wizard__mode-option">
              <input
                type="radio"
                name="creation-mode"
                value="fork"
                checked={creationModeChoice === 'fork'}
                onchange={() => selectCreationMode('fork')}
              />
              Fork
            </label>
          {/if}
          {#if capabilities.sourceHistoryMirrorEnabled || capabilities.sourceOnlyMirrorEnabled}
            <label class="wizard__mode-option">
              <input
                type="radio"
                name="creation-mode"
                value="mirror"
                checked={creationModeChoice === 'mirror'}
                onchange={() => selectCreationMode('mirror')}
              />
              Mirror
            </label>
          {/if}
        </div>
      {/if}

      {#if creationModeChoice === 'template'}
        <div class="wizard__step1-layout">
          <div class="wizard__step1-main">
            <TemplateGrid
              {templates}
              {selectedTemplateName}
              loading={templatesLoading}
              error={templatesError}
              ontemplateSelect={handleTemplateSelect}
              ontemplateDeselect={handleTemplateDeselect}
              onretry={loadTemplates}
            />

            <div class="wizard__actions wizard__actions--step1">
              <span class="wizard__btn-tip" title={step1NextTooltip ?? undefined}>
                <button
                  type="button"
                  class="wizard__btn-next"
                  disabled={!step1NextEnabled}
                  onclick={advanceStep}
                >
                  Next: Repository settings →
                </button>
              </span>
            </div>
          </div>
          <div class="wizard__step1-sidebar">
            <TemplateDetailsPanel
              {templateDetails}
              loading={templateDetailsLoading}
              error={templateDetailsError}
            />
          </div>
        </div>
      {:else}
        <div class="wizard__step2-fields">
          <div class="wizard__field">
            <label for="source-repository" class="wizard__label">Source repository</label>
            <input
              id="source-repository"
              type="text"
              class="wizard__input"
              placeholder="owner/repo"
              bind:value={sourceReference}
            />
            <p class="wizard__helper">
              Enter the "owner/repo" of the repository to {creationModeChoice}.
            </p>
          </div>

          {#if sourceReferenceError}
            <InlineAlert variant="error" message={sourceReferenceError} />
          {/if}
          {#if sourceOwnerWarning}
            <InlineAlert variant="info" message={sourceOwnerWarning} />
          {/if}

          {#if creationModeChoice === 'mirror' && mirrorHistoryToggleVisible}
            <div role="radiogroup" aria-label="History preservation" class="wizard__mode-choice">
              <label class="wizard__mode-option">
                <input
                  type="radio"
                  name="history-choice"
                  value="preserve"
                  checked={effectiveHistoryChoice === 'preserve'}
                  onchange={() => {
                    historyChoiceExplicit = 'preserve';
                  }}
                />
                Preserve full history
              </label>
              <label class="wizard__mode-option">
                <input
                  type="radio"
                  name="history-choice"
                  value="source-only"
                  checked={effectiveHistoryChoice === 'source-only'}
                  onchange={() => {
                    historyChoiceExplicit = 'source-only';
                  }}
                />
                Contents only (via pull request)
              </label>
            </div>
          {/if}

          <div class="wizard__actions wizard__actions--step1">
            <span class="wizard__btn-tip" title={step1NextTooltip ?? undefined}>
              <button
                type="button"
                class="wizard__btn-next"
                disabled={!step1NextEnabled}
                onclick={advanceStep}
              >
                Next: Repository settings →
              </button>
            </span>
          </div>
        </div>
      {/if}
    {:else if currentStep === 2}
      <h1 tabindex="-1" bind:this={wizardHeading} class="wizard__heading">Repository settings</h1>

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
              {#each teams as team (team.slug)}
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

        <RepositorySummary
          organization={data.organization}
          repositoryName={repoName}
          templateName={summaryTemplateLabel}
          typeName={selectedTypeName}
          {visibility}
          teamName={selectedTeamName}
        />
      </div>

      <div class="wizard__actions">
        <button type="button" class="wizard__btn-back" onclick={retreatStep}>← Back</button>
        <span class="wizard__btn-tip" title={step2ActionTooltip ?? undefined}>
          <button
            type="button"
            class="wizard__btn-next"
            disabled={!step2Actionable}
            onclick={handleStep2Action}
          >
            {hasVariables ? 'Next: Variables →' : 'Next: Review →'}
          </button>
        </span>
      </div>
    {:else if hasVariables && currentStep === variablesStepNumber}
      <h1 tabindex="-1" bind:this={wizardHeading} class="wizard__heading">Template variables</h1>

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
        <span class="wizard__btn-tip" title={step3NextTooltip ?? undefined}>
          <button
            type="button"
            class="wizard__btn-next"
            disabled={!step3NextEnabled}
            onclick={advanceStep}
          >
            Next: Review →
          </button>
        </span>
      </div>
    {:else if currentStep === reviewStepNumber}
      <h1 tabindex="-1" bind:this={wizardHeading} class="wizard__heading">Review and create</h1>

      {#if createError}
        <InlineAlert variant="error" message={createError} />
      {/if}

      <div class="wizard__step2-fields">
        <RepositorySummary
          organization={data.organization}
          repositoryName={repoName}
          templateName={templateDetails?.displayName ?? summaryTemplateLabel}
          typeName={selectedTypeName}
          {visibility}
          teamName={selectedTeamName}
          description={templateDetails?.description}
          tags={templateDetails?.tags ?? []}
          variables={reviewVariables}
        />

        {#if creationModeChoice !== 'template' && designatedTemplateName}
          <p class="wizard__designated-template">
            Settings template: <strong>{designatedTemplateName}</strong>
          </p>
        {/if}
      </div>

      <div class="wizard__actions">
        <button type="button" class="wizard__btn-back" onclick={retreatStep}>← Back</button>
        <button
          type="button"
          class="wizard__btn-create"
          disabled={creatingRepo}
          onclick={() => submitCreate(variableValues)}
        >
          Create Repository
        </button>
      </div>
    {/if}
  </div>

  <CreationOverlay visible={creatingRepo} />
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

  /* Widened on Step 1 only, to make room for the two-column
     template-grid/details-sidebar layout below. Settings/Variables/Review
     stay at the narrow 56rem width. */
  .wizard--step1 {
    max-width: 80rem;
  }

  .wizard__heading {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-text);
    margin: 0;
  }

  .wizard__heading:focus {
    outline: none;
  }

  .wizard__step1-layout {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .wizard__step1-main {
    flex: 1;
    min-width: 0;
  }

  .wizard__step1-sidebar {
    flex-shrink: 0;
  }

  /* Two-column layout once there is enough room for a 20rem sidebar
     alongside the template grid without cramping it. */
  @media (min-width: 60rem) {
    .wizard__step1-layout {
      flex-direction: row;
      align-items: flex-start;
    }

    .wizard__step1-sidebar {
      width: 20rem;
    }
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
    /* Allow pointer events to pass through to the wrapper span so title shows */
    pointer-events: none;
  }

  /* Wrapper that carries the tooltip title when the button is disabled */
  .wizard__btn-tip {
    display: inline-block;
  }
  .wizard__btn-tip:has(button:disabled) {
    cursor: not-allowed;
  }

  .wizard__btn-back {
    padding: 0.625rem 1rem;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    color: var(--color-text);
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .wizard__btn-back:hover {
    background-color: var(--color-surface);
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
    color: var(--color-text);
  }

  .wizard__select {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    background-color: var(--color-bg);
    color: var(--color-text);
  }

  .wizard__select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .wizard__helper {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
    margin: 0;
  }

  .wizard__input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    font-size: 0.9375rem;
    color: var(--color-text);
    background-color: var(--color-bg);
    box-sizing: border-box;
  }

  .wizard__input:focus {
    outline: 2px solid var(--brand-primary);
    outline-offset: 2px;
    border-color: var(--brand-primary);
  }

  /* Creation-mode / history-preservation radiogroups (WUI-005) */
  .wizard__mode-choice {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .wizard__mode-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9375rem;
    color: var(--color-text);
    cursor: pointer;
  }

  .wizard__designated-template {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    margin: 0;
  }
</style>
