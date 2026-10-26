import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import TemplateCard from '../src/lib/components/TemplateCard.svelte';
import type { RepositoryTypeBadge } from '../src/lib/components/TemplateCard.svelte';

const baseProps = {
  name: 'my-service-template',
  // Deliberately distinct from `name` so tests can prove the two fields are
  // rendered/used independently (#286 issue 2) rather than one being a copy
  // of the other.
  displayName: 'My Service Template (Human Readable)',
  description: 'A template for microservices',
  selected: false,
};

describe('TemplateCard (CMP-004)', () => {
  // -------------------------------------------------------------------------
  // Rendering
  // -------------------------------------------------------------------------

  // -------------------------------------------------------------------------
  // Human-readable display name (#286 issue 2)
  //
  // The card's heading must show `displayName`, not the repository slug
  // (`name`). `name` remains in use for selection semantics only (radio
  // `value`/`aria-label` below), so the two fields must be independently
  // wired rather than one being derived from the other.
  // -------------------------------------------------------------------------

  it('renders the displayName (not the slug name) as the card heading', () => {
    render(TemplateCard, { props: baseProps });
    expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent(baseProps.displayName);
  });

  it('does not render the raw slug name as the heading text when displayName differs', () => {
    render(TemplateCard, { props: baseProps });
    // Guards against a hardcoded/stub implementation that copies `name` into
    // the heading (or ignores `displayName` entirely): the heading's own
    // text must not equal the slug, even though the slug is deliberately a
    // substring-free, distinct value here.
    expect(screen.getByRole('heading', { level: 3 }).textContent?.trim()).not.toBe(baseProps.name);
  });

  it('still uses the slug `name` (not displayName) for the radio value, independent of the heading', () => {
    render(TemplateCard, { props: baseProps });
    expect(screen.getByRole('radio')).toHaveAttribute('value', baseProps.name);
  });

  it('renders a different heading when only displayName changes and name stays the same', () => {
    const { rerender } = render(TemplateCard, { props: baseProps });
    expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent(baseProps.displayName);

    rerender({ ...baseProps, displayName: 'Totally Different Display Name' });
    expect(screen.getByRole('heading', { level: 3 })).toHaveTextContent(
      'Totally Different Display Name',
    );
    // The radio's value must be unaffected by the displayName change.
    expect(screen.getByRole('radio')).toHaveAttribute('value', baseProps.name);
  });

  it('renders the template description', () => {
    render(TemplateCard, { props: baseProps });
    expect(screen.getByText('A template for microservices')).toBeInTheDocument();
  });

  it('renders up to 4 tags', () => {
    render(TemplateCard, {
      props: { ...baseProps, tags: ['typescript', 'node', 'api', 'rest'] },
    });
    expect(screen.getByText('typescript')).toBeInTheDocument();
    expect(screen.getByText('rest')).toBeInTheDocument();
  });

  it('does not render more than 4 tags when 5 are provided', () => {
    render(TemplateCard, {
      props: { ...baseProps, tags: ['a', 'b', 'c', 'd', 'fifth'] },
    });
    expect(screen.queryByText('fifth')).toBeNull();
  });

  it('renders the repository type badge when provided', () => {
    const badge: RepositoryTypeBadge = { typeName: 'service', policy: 'fixed' };
    render(TemplateCard, { props: { ...baseProps, repositoryTypeBadge: badge } });
    expect(screen.getByText('service')).toBeInTheDocument();
  });

  it('does not render a repository type badge when null', () => {
    const { container } = render(TemplateCard, {
      props: { ...baseProps, repositoryTypeBadge: null },
    });
    expect(container.querySelector('.template-card__type-badge')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Radio input semantics
  // -------------------------------------------------------------------------

  it('renders a radio input with name="template-selection"', () => {
    render(TemplateCard, { props: baseProps });
    expect(screen.getByRole('radio')).toHaveAttribute('name', 'template-selection');
  });

  it('radio input value equals the template name', () => {
    render(TemplateCard, { props: baseProps });
    expect(screen.getByRole('radio')).toHaveAttribute('value', 'my-service-template');
  });

  it('radio input is checked when selected=true', () => {
    render(TemplateCard, { props: { ...baseProps, selected: true } });
    expect(screen.getByRole('radio')).toBeChecked();
  });

  it('radio input is not checked when selected=false', () => {
    render(TemplateCard, { props: { ...baseProps, selected: false } });
    expect(screen.getByRole('radio')).not.toBeChecked();
  });

  it('radio input is wrapped in a label element', () => {
    render(TemplateCard, { props: baseProps });
    const radio = screen.getByRole('radio');
    expect(radio.closest('label')).toBeTruthy();
  });

  // -------------------------------------------------------------------------
  // Selected state
  // -------------------------------------------------------------------------

  it('shows a checkmark icon when selected=true', () => {
    const { container } = render(TemplateCard, { props: { ...baseProps, selected: true } });
    expect(container.querySelector('.template-card__checkmark')).toBeTruthy();
  });

  it('does not show a checkmark icon when selected=false', () => {
    const { container } = render(TemplateCard, { props: { ...baseProps, selected: false } });
    expect(container.querySelector('.template-card__checkmark')).toBeNull();
  });

  // -------------------------------------------------------------------------
  // Loading state
  // -------------------------------------------------------------------------

  it('does not render the template name when loading=true', () => {
    render(TemplateCard, { props: { ...baseProps, loading: true } });
    expect(screen.queryByText('my-service-template')).toBeNull();
  });

  it('does not render the displayName when loading=true', () => {
    render(TemplateCard, { props: { ...baseProps, loading: true } });
    expect(screen.queryByText(baseProps.displayName)).toBeNull();
    expect(screen.queryByRole('heading', { level: 3 })).toBeNull();
  });

  it('renders a loading skeleton element when loading=true', () => {
    const { container } = render(TemplateCard, { props: { ...baseProps, loading: true } });
    expect(container.querySelector('.template-card--loading')).toBeTruthy();
  });

  // -------------------------------------------------------------------------
  // Interaction
  // -------------------------------------------------------------------------

  it('fires the onselect callback when the radio input is clicked (unselected state)', async () => {
    const onselect = vi.fn();
    render(TemplateCard, { props: { ...baseProps, selected: false, onselect } });
    await fireEvent.click(screen.getByRole('radio'));
    expect(onselect).toHaveBeenCalledOnce();
  });

  it('fires the ondeselect callback when the radio is clicked while already selected', async () => {
    const ondeselect = vi.fn();
    render(TemplateCard, { props: { ...baseProps, selected: true, ondeselect } });
    await fireEvent.click(screen.getByRole('radio'));
    expect(ondeselect).toHaveBeenCalledOnce();
  });

  it('does not fire onselect when the radio is clicked while already selected', async () => {
    const onselect = vi.fn();
    render(TemplateCard, { props: { ...baseProps, selected: true, onselect } });
    await fireEvent.click(screen.getByRole('radio'));
    expect(onselect).not.toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // Description "Show more" / "Show less" toggle (#286 issue 1)
  //
  // The card has no way to see the full (CSS line-clamped) description
  // today. A `<button type="button">` toggle must let the user expand/
  // collapse it, must expose its state via `aria-expanded`, and — because
  // the whole card is a `<label>` wrapping a visually-hidden selection
  // radio — clicking the toggle must never select/deselect the card.
  //
  // NOTE (spec gap/assumption, flagged for the Tech Lead/Coder): the task
  // description does not name an exact selector for "removes/applies the
  // clamp". These tests assume the implementation follows the existing
  // BEM-ish naming convention already used for other description/card
  // modifiers (e.g. `.template-card--selected`, `.template-card__type-badge`)
  // and toggles a `template-card__description--expanded` modifier class on
  // the description <p>. If the Coder picks a different mechanism (e.g. an
  // inline style, a different class name, or a wrapper element), the single
  // test that asserts on that exact class name will need updating — the
  // aria-expanded/stopPropagation tests do not depend on this assumption.
  // -------------------------------------------------------------------------

  it('renders a "Show more" toggle button with aria-expanded="false" by default', () => {
    render(TemplateCard, { props: baseProps });
    const toggle = screen.getByRole('button', { name: /show more/i });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
  });

  it('renders the toggle button with type="button" so it cannot submit a form', () => {
    render(TemplateCard, { props: baseProps });
    expect(screen.getByRole('button', { name: /show more/i })).toHaveAttribute('type', 'button');
  });

  it('expands and relabels the toggle to "Show less" with aria-expanded="true" on click', async () => {
    render(TemplateCard, { props: baseProps });
    await fireEvent.click(screen.getByRole('button', { name: /show more/i }));
    const toggle = screen.getByRole('button', { name: /show less/i });
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
  });

  it('collapses back to "Show more" with aria-expanded="false" on a second click', async () => {
    render(TemplateCard, { props: baseProps });
    const firstClick = screen.getByRole('button', { name: /show more/i });
    await fireEvent.click(firstClick);
    const secondClick = screen.getByRole('button', { name: /show less/i });
    await fireEvent.click(secondClick);
    const toggle = screen.getByRole('button', { name: /show more/i });
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
  });

  it('adds a description "expanded" modifier class when toggled open', async () => {
    const { container } = render(TemplateCard, { props: baseProps });
    await fireEvent.click(screen.getByRole('button', { name: /show more/i }));
    const description = container.querySelector('.template-card__description');
    expect(description?.classList.contains('template-card__description--expanded')).toBe(true);
  });

  it('removes the description "expanded" modifier class when toggled closed again', async () => {
    const { container } = render(TemplateCard, { props: baseProps });
    await fireEvent.click(screen.getByRole('button', { name: /show more/i }));
    await fireEvent.click(screen.getByRole('button', { name: /show less/i }));
    const description = container.querySelector('.template-card__description');
    expect(description?.classList.contains('template-card__description--expanded')).toBe(false);
  });

  it('does not fire onselect when the "Show more" toggle is clicked (unselected card)', async () => {
    const onselect = vi.fn();
    render(TemplateCard, { props: { ...baseProps, selected: false, onselect } });
    await fireEvent.click(screen.getByRole('button', { name: /show more/i }));
    expect(onselect).not.toHaveBeenCalled();
  });

  it('does not fire ondeselect when the "Show more" toggle is clicked (selected card)', async () => {
    const ondeselect = vi.fn();
    render(TemplateCard, { props: { ...baseProps, selected: true, ondeselect } });
    await fireEvent.click(screen.getByRole('button', { name: /show more/i }));
    expect(ondeselect).not.toHaveBeenCalled();
  });

  it('does not check the selection radio when the "Show more" toggle is clicked', async () => {
    render(TemplateCard, { props: { ...baseProps, selected: false } });
    await fireEvent.click(screen.getByRole('button', { name: /show more/i }));
    expect(screen.getByRole('radio')).not.toBeChecked();
  });

  it('keeps the toggle button nested inside the selection label, not detached from it', () => {
    render(TemplateCard, { props: baseProps });
    const toggle = screen.getByRole('button', { name: /show more/i });
    expect(toggle.closest('label')).toBeTruthy();
  });
});
