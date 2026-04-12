<script lang="ts">
  /**
   * CMP-003: StepProgress
   *
   * Visual step progress indicator for the multi-step creation wizard.
   * Does not steal focus — focus management is the parent wizard's responsibility.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-003
   * Assertion: UX-ASSERT-024
   */

  interface Props {
    steps: string[];
    currentStep: number; // 1-indexed
  }

  let { steps, currentStep }: Props = $props();
</script>

<div
  class="step-progress"
  role="group"
  aria-label="Step {currentStep} of {steps.length}: {steps[currentStep - 1]}"
>
  <ol class="step-progress__list">
    {#each steps as step, i}
      {@const stepNum = i + 1}
      {@const isCompleted = stepNum < currentStep}
      {@const isCurrent = stepNum === currentStep}
      <li class="step-progress__item">
        <span
          role="img"
          aria-label={isCompleted
            ? `${step}, completed`
            : isCurrent
              ? `${step}, current step`
              : step}
          class="step-progress__dot"
          class:step-progress__dot--completed={isCompleted}
          class:step-progress__dot--current={isCurrent}
          class:step-progress__dot--upcoming={!isCompleted && !isCurrent}
        >
          {#if isCompleted}
            <span aria-hidden="true">✓</span>
          {:else}
            <span aria-hidden="true">{stepNum}</span>
          {/if}
        </span>
        <span class="step-progress__label" aria-hidden="true">{step}</span>
      </li>
      {#if i < steps.length - 1}
        <li aria-hidden="true" class="step-progress__connector"></li>
      {/if}
    {/each}
  </ol>
</div>

<style>
  .step-progress {
    padding: 0.5rem 0;
  }

  .step-progress__list {
    display: flex;
    align-items: center;
    list-style: none;
    padding: 0;
    margin: 0;
    gap: 0;
  }

  .step-progress__item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .step-progress__connector {
    flex: 1;
    height: 2px;
    background-color: var(--color-border);
    margin: 0 0.5rem;
    min-width: 2rem;
  }

  .step-progress__dot {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    font-size: 0.875rem;
    font-weight: 600;
    flex-shrink: 0;
  }

  .step-progress__dot--completed {
    background-color: var(--brand-primary, #2563eb);
    color: #fff;
  }

  .step-progress__dot--current {
    background-color: var(--brand-primary, #2563eb);
    color: #fff;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.25);
  }

  .step-progress__dot--upcoming {
    background-color: var(--color-border);
    color: var(--color-text-muted);
  }

  .step-progress__label {
    font-size: 0.875rem;
    color: var(--color-text-muted);
    white-space: nowrap;
  }
</style>
