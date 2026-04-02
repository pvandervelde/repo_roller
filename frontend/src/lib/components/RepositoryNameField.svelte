<script lang="ts">
  /**
   * CMP-006: RepositoryNameField
   *
   * Controlled text input for repository name with client-side format validation
   * (debounced 300ms) and async uniqueness checking on blur.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-006
   * Assertions: UX-ASSERT-009, UX-ASSERT-010, UX-ASSERT-011, UX-ASSERT-012,
   *             UX-ASSERT-013, UX-ASSERT-028
   */
  import { validateRepositoryName } from '$lib/api/client';

  export type NameValidationResult =
    | { status: 'idle' }
    | { status: 'invalid_format'; message: string }
    | { status: 'checking' }
    | { status: 'available' }
    | { status: 'taken'; name: string }
    | { status: 'check_failed' };

  /** Mirrors GitHub name rules; also validated server-side. */
  const NAME_RE = /^[a-z0-9._-]+$/;

  function validateFormat(name: string): NameValidationResult {
    if (!name) return { status: 'idle' };
    if (!NAME_RE.test(name)) {
      return {
        status: 'invalid_format',
        message:
          'Repository names may only contain lowercase letters, numbers, hyphens (-), underscores (_), and dots (.). Names cannot start with a dot.',
      };
    }
    if (name.startsWith('.')) {
      return {
        status: 'invalid_format',
        message:
          'Repository names may only contain lowercase letters, numbers, hyphens (-), underscores (_), and dots (.). Names cannot start with a dot.',
      };
    }
    if (name.includes('..')) {
      return {
        status: 'invalid_format',
        message:
          'Repository names may only contain lowercase letters, numbers, hyphens (-), underscores (_), and dots (.). Names cannot start with a dot.',
      };
    }
    return { status: 'idle' };
  }

  interface Props {
    value: string;
    organization: string;
    disabled?: boolean;
    onchange?: (value: string) => void;
    onvalidationResult?: (result: NameValidationResult) => void;
  }

  let { value, organization, disabled = false, onchange, onvalidationResult }: Props = $props();

  let validation: NameValidationResult = $state({ status: 'idle' });
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  const statusId = 'repo-name-status';

  function handleInput(e: Event) {
    const newValue = (e.target as HTMLInputElement).value;
    onchange?.(newValue);

    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      const result = validateFormat(newValue);
      if (result.status !== 'idle') {
        validation = result;
        onvalidationResult?.(result);
      } else if (validation.status === 'taken' || validation.status === 'check_failed') {
        // Clear taken/check_failed on any keystroke  UX-ASSERT-013
        validation = { status: 'idle' };
        onvalidationResult?.({ status: 'idle' });
      } else if (validation.status === 'invalid_format') {
        validation = { status: 'idle' };
        onvalidationResult?.({ status: 'idle' });
      }
    }, 300);

    // Immediately clear "taken" on any keystroke  UX-ASSERT-013
    if (validation.status === 'taken') {
      validation = { status: 'idle' };
      onvalidationResult?.({ status: 'idle' });
    }
  }

  async function handleBlur() {
    if (debounceTimer) {
      clearTimeout(debounceTimer);
      debounceTimer = null;
    }

    const currentValue = value;
    if (!currentValue) return;

    const formatResult = validateFormat(currentValue);
    if (formatResult.status === 'invalid_format') {
      validation = formatResult;
      onvalidationResult?.(formatResult);
      return;
    }

    // Good format — check uniqueness  UX-ASSERT-010
    validation = { status: 'checking' };
    onvalidationResult?.({ status: 'checking' });

    try {
      const result = await validateRepositoryName(organization, currentValue);
      if (result.valid) {
        validation = { status: 'available' };
        onvalidationResult?.({ status: 'available' });
      } else {
        validation = { status: 'taken', name: currentValue };
        onvalidationResult?.({ status: 'taken', name: currentValue });
      }
    } catch {
      validation = { status: 'check_failed' };
      onvalidationResult?.({ status: 'check_failed' });
    }
  }
</script>

<div class="repo-name-field">
  <label for="repository-name" class="repo-name-field__label">Repository name</label>
  <input
    id="repository-name"
    type="text"
    class="repo-name-field__input"
    class:repo-name-field__input--error={validation.status === 'invalid_format' ||
      validation.status === 'taken'}
    class:repo-name-field__input--available={validation.status === 'available'}
    {value}
    placeholder="e.g. my-new-service"
    {disabled}
    aria-describedby={statusId}
    oninput={handleInput}
    onblur={handleBlur}
  />
  <p class="repo-name-field__helper">
    Lowercase letters, numbers, hyphens, underscores, and dots. Must be unique in the organization.
    Cannot start with a dot.
  </p>

  <div id={statusId} class="repo-name-field__status" aria-live="polite">
    {#if validation.status === 'checking'}
      <span role="status" class="repo-name-field__checking">Checking availability…</span>
    {:else if validation.status === 'available'}
      <span class="repo-name-field__available">Available</span>
    {:else if validation.status === 'taken'}
      <span role="alert" class="repo-name-field__error">
        '{value}' is already taken in this organization.
      </span>
    {:else if validation.status === 'invalid_format'}
      <span role="alert" class="repo-name-field__error">{validation.message}</span>
    {:else if validation.status === 'check_failed'}
      <span role="status" class="repo-name-field__warning">
        Could not check availability. You can still proceed, but the name may already exist.
      </span>
    {/if}
  </div>
</div>

<style>
  .repo-name-field {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .repo-name-field__label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .repo-name-field__input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: #111827;
    background-color: #fff;
    box-sizing: border-box;
    transition: border-color 0.15s;
  }

  .repo-name-field__input:focus {
    outline: 2px solid var(--brand-primary, #2563eb);
    outline-offset: 2px;
    border-color: var(--brand-primary, #2563eb);
  }

  .repo-name-field__input:disabled {
    background-color: #f9fafb;
    color: #9ca3af;
    cursor: not-allowed;
  }

  .repo-name-field__input--error {
    border-color: #dc2626;
  }

  .repo-name-field__input--available {
    border-color: #16a34a;
  }

  .repo-name-field__helper {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  .repo-name-field__status {
    min-height: 1.25rem;
    font-size: 0.8125rem;
  }

  .repo-name-field__checking {
    color: #6b7280;
  }

  .repo-name-field__available {
    color: #16a34a;
    font-weight: 500;
  }

  .repo-name-field__error {
    color: #dc2626;
  }

  .repo-name-field__warning {
    color: #b45309;
  }
</style>
