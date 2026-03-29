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
</script>

<!-- TODO: implement -->
