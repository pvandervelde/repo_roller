<script lang="ts">
  /**
   * CMP-002: UserBadge
   *
   * Displays the authenticated user's GitHub avatar and username.
   * Provides a dropdown with a sign-out trigger.
   * Uses Svelte 5 callback prop pattern for the signOut event.
   *
   * Spec: docs/spec/ux/components/component-inventory.md — CMP-002
   * Assertion: UX-ASSERT-004
   */
  interface Props {
    login: string;
    avatarUrl?: string | null;
    onsignOut?: () => void;
  }

  let { login, avatarUrl = null, onsignOut }: Props = $props();

  let open = $state(false);

  function toggleDropdown() {
    open = !open;
  }

  function handleSignOut() {
    open = false;
    onsignOut?.();
  }
</script>

<div class="user-badge">
  <button
    type="button"
    class="user-badge__toggle"
    aria-haspopup="true"
    aria-expanded={open ? 'true' : 'false'}
    onclick={toggleDropdown}
  >
    {#if avatarUrl}
      <img src={avatarUrl} alt="{login}'s GitHub avatar" class="user-badge__avatar" />
    {:else}
      <span class="user-badge__initials" aria-hidden="true">{login[0].toUpperCase()}</span>
    {/if}
    <span class="user-badge__login">{login}</span>
  </button>

  {#if open}
    <ul role="menu" class="user-badge__menu">
      <li role="none">
        <button type="button" role="menuitem" class="user-badge__menu-item" onclick={handleSignOut}>
          Sign out
        </button>
      </li>
    </ul>
  {/if}
</div>

<style>
  .user-badge {
    position: relative;
    display: inline-block;
  }

  .user-badge__toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: none;
    border: none;
    padding: 0.25rem 0.5rem;
    cursor: pointer;
    border-radius: 0.25rem;
  }

  .user-badge__toggle:hover {
    background-color: rgba(0, 0, 0, 0.05);
  }

  .user-badge__avatar {
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    object-fit: cover;
  }

  .user-badge__initials {
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    background-color: var(--brand-primary, #2563eb);
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.875rem;
    font-weight: 600;
  }

  .user-badge__menu {
    position: absolute;
    right: 0;
    top: calc(100% + 0.25rem);
    min-width: 10rem;
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.25rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
    z-index: 100;
  }

  .user-badge__menu-item {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 0.875rem;
    color: #374151;
  }

  .user-badge__menu-item:hover {
    background-color: #f3f4f6;
  }
</style>
