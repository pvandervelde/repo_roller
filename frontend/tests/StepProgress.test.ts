import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import StepProgress from '../src/lib/components/StepProgress.svelte';

const twoSteps = ['Choose template', 'Settings'];
const threeSteps = ['Choose template', 'Settings', 'Variables'];

describe('StepProgress (CMP-003)', () => {
  // -------------------------------------------------------------------------
  // Wrapper aria-label
  // -------------------------------------------------------------------------

  it('wrapper has correct aria-label for step 1 of 2', () => {
    render(StepProgress, { props: { steps: twoSteps, currentStep: 1 } });
    expect(screen.getByRole('group', { name: 'Step 1 of 2: Choose template' })).toBeInTheDocument();
  });

  it('wrapper aria-label reflects current step when on step 2 of 2', () => {
    render(StepProgress, { props: { steps: twoSteps, currentStep: 2 } });
    expect(screen.getByRole('group', { name: 'Step 2 of 2: Settings' })).toBeInTheDocument();
  });

  it('wrapper aria-label reflects a 3-step flow', () => {
    render(StepProgress, { props: { steps: threeSteps, currentStep: 3 } });
    expect(screen.getByRole('group', { name: 'Step 3 of 3: Variables' })).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Step labels
  // -------------------------------------------------------------------------

  it('renders all step labels', () => {
    render(StepProgress, { props: { steps: threeSteps, currentStep: 1 } });
    expect(screen.getByText('Choose template')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByText('Variables')).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Step dot aria-labels
  // -------------------------------------------------------------------------

  it('completed step dot has aria-label containing "completed"', () => {
    render(StepProgress, { props: { steps: threeSteps, currentStep: 2 } });
    const completedDot = screen.getByRole('img', { name: /choose template.*completed/i });
    expect(completedDot).toBeInTheDocument();
  });

  it('current step dot has aria-label containing "current step"', () => {
    render(StepProgress, { props: { steps: threeSteps, currentStep: 2 } });
    const currentDot = screen.getByRole('img', { name: /settings.*current step/i });
    expect(currentDot).toBeInTheDocument();
  });

  it('upcoming step dot has plain aria-label (no completed or current suffix)', () => {
    render(StepProgress, { props: { steps: threeSteps, currentStep: 2 } });
    // "Variables" step is upcoming — should be accessible but not labelled completed or current
    expect(screen.getByText('Variables')).toBeInTheDocument();
    expect(screen.queryByRole('img', { name: /variables.*completed/i })).toBeNull();
    expect(screen.queryByRole('img', { name: /variables.*current/i })).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Completed step visual
  // -------------------------------------------------------------------------

  it('shows a checkmark indicator for completed steps', () => {
    const { container } = render(StepProgress, {
      props: { steps: threeSteps, currentStep: 3 },
    });
    const completedDots = container.querySelectorAll('.step-progress__dot--completed');
    expect(completedDots.length).toBe(2); // steps 1 and 2
  });

  it('marks exactly one dot as current', () => {
    const { container } = render(StepProgress, {
      props: { steps: threeSteps, currentStep: 2 },
    });
    expect(container.querySelectorAll('.step-progress__dot--current').length).toBe(1);
  });
});
