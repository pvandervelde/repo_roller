import { test, expect } from '@playwright/test';

/**
 * E2E tests for the authentication flow.
 * Covers: SCR-001 (sign-in), SCR-003 (access denied), SCR-006 (error screen),
 * route guards (UX-ASSERT-001), and session-based access.
 *
 * NOTE: Real GitHub OAuth cannot be exercised in automated tests without live credentials.
 * The OAuth callback path (SCR-002) is omitted here and covered by unit tests in tests/.
 */

// A minimal valid session cookie for testing authenticated pages.
const SESSION_COOKIE = {
  name: 'session',
  value: JSON.stringify({
    userLogin: 'test-user',
    userAvatarUrl: 'https://example.com/avatar.png',
  }),
  domain: 'localhost',
  path: '/',
  httpOnly: true,
  secure: false,
  sameSite: 'Lax' as const,
};

// ---------------------------------------------------------------------------
// SCR-001: Sign-in page
// ---------------------------------------------------------------------------

test.describe('Sign-in page (SCR-001)', () => {
  test('page title contains "Sign in"', async ({ page }) => {
    await page.goto('/sign-in');
    await expect(page).toHaveTitle(/Sign in/);
  });

  test('renders an h1 heading', async ({ page }) => {
    await page.goto('/sign-in');
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  });

  test('renders "Sign in with GitHub" link-button', async ({ page }) => {
    await page.goto('/sign-in');
    await expect(page.getByRole('button', { name: /Sign in with GitHub/i })).toBeVisible();
  });

  test('"Sign in with GitHub" button links to GitHub OAuth', async ({ page }) => {
    await page.goto('/sign-in');
    const btn = page.getByRole('button', { name: /Sign in with GitHub/i });
    const href = await btn.getAttribute('href');
    expect(href).toContain('github.com/login/oauth/authorize');
  });
});

// ---------------------------------------------------------------------------
// Route guards (UX-ASSERT-001)
// ---------------------------------------------------------------------------

test.describe('Route guards (UX-ASSERT-001)', () => {
  test('unauthenticated /create redirects to /sign-in', async ({ page }) => {
    await page.goto('/create');
    await expect(page).toHaveURL('/sign-in');
  });

  test('unauthenticated /create/success redirects to /sign-in', async ({ page }) => {
    await page.goto('/create/success');
    await expect(page).toHaveURL('/sign-in');
  });

  test('session cookie allows access to /create (UX-ASSERT-001)', async ({ page, context }) => {
    await context.addCookies([SESSION_COOKIE]);
    await page.goto('/create');
    await expect(page).not.toHaveURL('/sign-in');
  });
});

// ---------------------------------------------------------------------------
// SCR-003: Access Denied page
// ---------------------------------------------------------------------------

test.describe('Access Denied page (SCR-003)', () => {
  test('access_denied reason shows "GitHub authorization was cancelled"', async ({ page }) => {
    await page.goto('/auth/denied?reason=access_denied');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(
      'GitHub authorization was cancelled',
    );
  });

  test('oauth_error reason shows "Sign-in could not be completed"', async ({ page }) => {
    await page.goto('/auth/denied?reason=oauth_error');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(
      'Sign-in could not be completed',
    );
  });

  test('no reason defaults to oauth_error copy', async ({ page }) => {
    await page.goto('/auth/denied');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(
      'Sign-in could not be completed',
    );
  });

  test('"Try again" button navigates to /sign-in', async ({ page }) => {
    await page.goto('/auth/denied?reason=oauth_error');
    const btn = page.getByRole('link', { name: 'Try again' });
    await expect(btn).toHaveAttribute('href', '/sign-in');
  });
});

// ---------------------------------------------------------------------------
// SCR-006: Error screen
// ---------------------------------------------------------------------------

test.describe('Error screen (SCR-006)', () => {
  test('generic error shows "Something went wrong"', async ({ page }) => {
    await page.goto('/error');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('Something went wrong');
  });

  test('session_expired shows "Your session has expired"', async ({ page }) => {
    await page.goto('/error?reason=session_expired');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('Your session has expired');
  });

  test('generic error message uses role="alert"', async ({ page }) => {
    await page.goto('/error');
    await expect(page.getByRole('alert')).toContainText(
      'An unexpected error occurred. If this keeps happening, contact your platform team.',
    );
  });

  test('"Try again" button links to /sign-in when no session', async ({ page }) => {
    await page.goto('/error');
    await expect(page.getByRole('link', { name: 'Try again' })).toHaveAttribute('href', '/sign-in');
  });

  test('"Sign in" button for session_expired links to /sign-in', async ({ page }) => {
    await page.goto('/error?reason=session_expired');
    await expect(page.getByRole('link', { name: 'Sign in' })).toHaveAttribute('href', '/sign-in');
  });
});

// ---------------------------------------------------------------------------
// Sign-out flow (UX-ASSERT-004)
// ---------------------------------------------------------------------------

test.describe('Sign-out flow (UX-ASSERT-004)', () => {
  test('sign-out destroys session and redirects to /sign-in', async ({ page, context }) => {
    // Inject a valid session cookie so the user starts authenticated.
    await context.addCookies([SESSION_COOKIE]);

    // Confirm the user can reach the authenticated /create route.
    await page.goto('/create');
    await expect(page).not.toHaveURL('/sign-in');

    // Click the "Sign out" menu item in the UserBadge dropdown.
    await page.getByRole('button', { name: /sign out/i }).click();

    // After sign-out the server clears the cookie and redirects to /sign-in.
    await expect(page).toHaveURL('/sign-in');
  });

  test('after sign-out, navigating to /create redirects to /sign-in', async ({ page, context }) => {
    // Start authenticated.
    await context.addCookies([SESSION_COOKIE]);
    await page.goto('/create');
    await expect(page).not.toHaveURL('/sign-in');

    // Sign out.
    await page.getByRole('button', { name: /sign out/i }).click();
    await expect(page).toHaveURL('/sign-in');

    // Attempting to navigate back to /create must redirect to /sign-in because
    // the session cookie was cleared by the sign-out server action.
    await page.goto('/create');
    await expect(page).toHaveURL('/sign-in');
  });
});
