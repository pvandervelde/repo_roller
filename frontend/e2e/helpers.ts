import { createHmac } from 'node:crypto';
import type { Cookie } from '@playwright/test';

/**
 * Session secret used for E2E tests.
 *
 * MUST match the SESSION_SECRET value set on the webServer in
 * playwright.config.ts. It is a fixed test-only value and MUST
 * never be used in production.
 */
export const E2E_SESSION_SECRET = 'e2e-test-session-secret-do-not-use-in-production';

/**
 * Sign a session payload using the same HMAC-SHA256 format as
 * src/lib/session.server.ts, so the server's parseSessionCookie()
 * will accept it:
 *
 *   <base64url(JSON.stringify(payload))>.<hex(hmac-sha256(data, secret))>
 */
export function signSessionCookieValue(payload: object): string {
  const data = Buffer.from(JSON.stringify(payload)).toString('base64url');
  const sig = createHmac('sha256', E2E_SESSION_SECRET).update(data).digest('hex');
  return `${data}.${sig}`;
}

/**
 * Build a properly signed session cookie for a synthetic test user.
 * Pass the result directly to `context.addCookies()` in Playwright tests.
 */
export function makeSessionCookie(): Cookie {
  return {
    name: 'session',
    value: signSessionCookieValue({
      userLogin: 'test-user',
      userAvatarUrl: 'https://example.com/avatar.png',
    }),
    domain: 'localhost',
    path: '/',
    httpOnly: true,
    secure: false,
    sameSite: 'Lax',
  };
}
