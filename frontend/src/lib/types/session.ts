/**
 * Authenticated session for the current user.
 * Stored in an HTTP-only, Secure, SameSite=Lax cookie as JSON.
 */
export interface Session {
  /** GitHub login (username) of the authenticated user. */
  userLogin: string;

  /** GitHub avatar URL. May be null if unavailable. */
  userAvatarUrl: string | null;

  /**
   * Backend-signed JWT issued by POST /api/v1/auth/token.
   * Used as the Authorization: Bearer value on all backend API requests.
   * Expires after 8 hours (matches the session cookie lifetime).
   */
  backendToken: string;
}
