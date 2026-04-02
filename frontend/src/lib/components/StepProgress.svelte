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

<div class="step-progress"></div>
