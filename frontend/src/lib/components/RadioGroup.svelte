<script lang="ts">
  /**
   * RadioGroup
   *
   * Shared, minimal `role="radiogroup"` control for a flat list of labelled
   * text options (as opposed to `TemplateGrid`/`TemplateCard`'s visual card
   * radiogroup, which has its own distinct layout and is not a fit for this
   * component).
   *
   * Extracted from two verbatim-duplicated inline blocks in
   * `routes/create/+page.svelte` — the "Creation mode" (Template/Fork/Mirror)
   * and "History preservation" (Preserve full history/Contents only)
   * radiogroups.
   */
  export interface RadioGroupOption {
    value: string;
    label: string;
  }

  interface Props {
    ariaLabel: string;
    name: string;
    options: RadioGroupOption[];
    selected: string | null;
    onchange: (value: string) => void;
  }

  let { ariaLabel, name, options, selected, onchange }: Props = $props();
</script>

<div role="radiogroup" aria-label={ariaLabel} class="radio-group">
  {#each options as option (option.value)}
    <label class="radio-group__option">
      <input
        type="radio"
        {name}
        value={option.value}
        checked={selected === option.value}
        onchange={() => onchange(option.value)}
      />
      {option.label}
    </label>
  {/each}
</div>

<style>
  .radio-group {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .radio-group__option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9375rem;
    color: var(--color-text);
    cursor: pointer;
  }
</style>
