import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Spinner from '../src/lib/components/Spinner.svelte';

// ---------------------------------------------------------------------------
// RED-phase spec-gap notes (Tester → Coder):
//
// `Spinner.svelte` does not exist yet. It is being extracted from
// `CreationOverlay.svelte`'s existing `.creation-overlay__spinner` +
// `@keyframes spin` (0.8s linear infinite) — see CreationOverlay.test.ts for
// the additive assertions proving the extraction happened without behaviour
// change.
//
// The Interface Contract only specifies the prop shape
// (`size?: 'sm' | 'md' | 'lg'`, `label?: string`), not exact class names or
// the default size. This file documents the choices the tests assume, so the
// Coder can either match them or update the handful of tests that assert on
// the literal strings below (same latitude granted to the #286 RED phase for
// its documented spec gaps):
//
//   - Root element carries class "spinner" always, plus exactly one of
//     "spinner--sm" / "spinner--md" / "spinner--lg" depending on `size`.
//   - Default size when `size` is omitted is "md".
//   - When `label` is provided, the root element has role="status" and
//     aria-label={label} (standalone, self-announcing spinner).
//   - When `label` is omitted, the root element has aria-hidden="true" and is
//     NOT exposed via role="status" — this matters because
//     `TemplateDetailsPanel` wraps its own role="status"/aria-live="polite"
//     region around a label-less Spinner + text; a self-announcing Spinner
//     nested inside that region would cause duplicate screen-reader
//     announcements.
// ---------------------------------------------------------------------------

describe('Spinner (new component — extracted from CreationOverlay)', () => {
  // -------------------------------------------------------------------------
  // Stub-killing: renders *something* visual, not an empty fragment
  // -------------------------------------------------------------------------

  it('renders a single element carrying the "spinner" class', () => {
    const { container } = render(Spinner, { props: {} });
    const spinners = container.querySelectorAll('.spinner');
    expect(spinners.length).toBe(1);
  });

  // -------------------------------------------------------------------------
  // Size variants — kills a hardcoded-"lg" stub (CreationOverlay currently
  // hardcodes 3rem/"lg"; the whole point of extraction is parameterizing it)
  // -------------------------------------------------------------------------

  it('defaults to the "md" size modifier when no size prop is given', () => {
    const { container } = render(Spinner, { props: {} });
    expect(container.querySelector('.spinner--md')).toBeTruthy();
  });

  it('applies the "sm" size modifier when size="sm"', () => {
    const { container } = render(Spinner, { props: { size: 'sm' } });
    expect(container.querySelector('.spinner--sm')).toBeTruthy();
  });

  it('applies the "md" size modifier when size="md"', () => {
    const { container } = render(Spinner, { props: { size: 'md' } });
    expect(container.querySelector('.spinner--md')).toBeTruthy();
  });

  it('applies the "lg" size modifier when size="lg" (matches CreationOverlay\'s current 3rem spinner)', () => {
    const { container } = render(Spinner, { props: { size: 'lg' } });
    expect(container.querySelector('.spinner--lg')).toBeTruthy();
  });

  it('does not apply more than one size modifier at a time', () => {
    const { container } = render(Spinner, { props: { size: 'sm' } });
    const hasMd = container.querySelector('.spinner--md') !== null;
    const hasLg = container.querySelector('.spinner--lg') !== null;
    expect(hasMd).toBe(false);
    expect(hasLg).toBe(false);
  });

  it('updates the size modifier class when the size prop changes (kills a hardcoded size stub)', async () => {
    const { container, rerender } = render(Spinner, { props: { size: 'sm' } });
    expect(container.querySelector('.spinner--sm')).toBeTruthy();

    await rerender({ size: 'lg' });
    expect(container.querySelector('.spinner--lg')).toBeTruthy();
    expect(container.querySelector('.spinner--sm')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Accessibility — label present vs omitted
  // -------------------------------------------------------------------------

  it('exposes role="status" with the given aria-label when label is provided', () => {
    render(Spinner, { props: { label: 'Loading template details' } });
    expect(
      screen.getByRole('status', { name: 'Loading template details' }),
    ).toBeInTheDocument();
  });

  it('uses a different aria-label text verbatim when label changes (kills a hardcoded label stub)', () => {
    render(Spinner, { props: { label: 'Creating repository' } });
    expect(screen.getByRole('status', { name: 'Creating repository' })).toBeInTheDocument();
    expect(screen.queryByRole('status', { name: 'Loading template details' })).toBeNull();
  });

  it('does not expose role="status" when label is omitted', () => {
    render(Spinner, { props: {} });
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('carries aria-hidden="true" on the root element when label is omitted', () => {
    const { container } = render(Spinner, { props: { size: 'md' } });
    const root = container.querySelector('.spinner');
    expect(root).toHaveAttribute('aria-hidden', 'true');
  });

  it('does not render aria-hidden="true" on the root element when a label is provided', () => {
    render(Spinner, { props: { label: 'Creating repository' } });
    const root = screen.getByRole('status', { name: 'Creating repository' });
    expect(root).not.toHaveAttribute('aria-hidden', 'true');
  });

  // -------------------------------------------------------------------------
  // Robustness
  // -------------------------------------------------------------------------

  it('does not throw and renders the default-size spinner when called with zero props', () => {
    expect(() => render(Spinner, { props: {} })).not.toThrow();
  });
});
