/**
 * Authenticated session for the current user.
 * Stored in an HTTP-only, Secure, SameSite=Lax cookie as JSON.
 */
export interface Session {
  /** GitHub login (username) of the authenticated user. */
  userLogin: string;

  /** GitHub avatar URL. May be null if unavailable. */
  userAvatarUrl: string | null;
}
