import { test, expect, type BrowserContext } from '@playwright/test';
import { makeSessionCookie } from './helpers';

/**
 * E2E tests for the repository creation flow.
 * Covers: SCR-004 (wizard), SCR-005 (success screen).
 *
 * All API calls are intercepted via page.route() to avoid requiring a real backend.
 * Authentication is simulated by setting a session cookie before each test.
 */

const MOCK_TEMPLATES = [
  {
    name: 'basic-service',
    description: 'A basic service template',
    tags: ['service'],
    repository_type_name: 'service',
    repository_type_policy: 'preferable',
    variable_count: 0,
  },
  {
    name: 'data-pipeline',
    description: 'A data pipeline template',
    tags: ['data'],
    repository_type_name: null,
    repository_type_policy: 'optional',
    variable_count: 2,
  },
];

const MOCK_TEMPLATE_DETAIL_NO_VARS = {
  name: 'basic-service',
  description: 'A basic service template',
  tags: ['service'],
  repository_type_name: 'service',
  repository_type_policy: 'preferable' as const,
  variables: [],
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function addSessionCookie(context: BrowserContext) {
  await context.addCookies([makeSessionCookie()]);
}

// ---------------------------------------------------------------------------
// SCR-004: Creation wizard — Step 1
// ---------------------------------------------------------------------------

test.describe('Creation wizard — Step 1: Choose a template (SCR-004)', () => {
  test.beforeEach(async ({ context }) => {
    await addSessionCookie(context);
  });

  test('wizard heading is visible', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });

    await page.goto('/create');
    await expect(page.getByRole('heading', { name: /choose a template/i })).toBeVisible();
  });

  test('search box is rendered (UX-ASSERT-005)', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });

    await page.goto('/create');
    await expect(page.getByRole('searchbox', { name: /search templates/i })).toBeVisible();
  });

  test('template cards render after API success', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });

    await page.goto('/create');
    await expect(page.getByText('basic-service')).toBeVisible();
    await expect(page.getByText('data-pipeline')).toBeVisible();
  });

  test('API error shows retry button (UX-ASSERT-008)', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({ status: 500, body: '' });
    });

    await page.goto('/create');
    await expect(page.getByRole('button', { name: /try again/i })).toBeVisible();
  });

  test('"Next" button is disabled before template selection (UX-ASSERT-006)', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates\/basic-service$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_TEMPLATE_DETAIL_NO_VARS),
      });
    });

    await page.goto('/create');
    // Wait for templates to load
    await expect(page.getByText('basic-service')).toBeVisible();
    // Next button should be disabled before selection
    const nextBtn = page.getByRole('button', { name: /next/i });
    await expect(nextBtn).toBeDisabled();
  });

  test('selecting a template activates "Next" button (UX-ASSERT-006)', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates\/basic-service$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_TEMPLATE_DETAIL_NO_VARS),
      });
    });

    await page.goto('/create');
    await expect(page.getByText('basic-service')).toBeVisible();
    await page.getByRole('heading', { level: 3, name: 'basic-service' }).click();
    await expect(page.getByRole('button', { name: /next/i })).toBeEnabled();
  });
});

// ---------------------------------------------------------------------------
// SCR-004: Step 2 — Repository settings
// ---------------------------------------------------------------------------

test.describe('Creation wizard — Step 2: Repository settings (SCR-004)', () => {
  test.beforeEach(async ({ context }) => {
    await addSessionCookie(context);
  });

  test('Step 2 renders after advancing from Step 1', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates\/basic-service$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_TEMPLATE_DETAIL_NO_VARS),
      });
    });
    await page.route(/\/api\/v1\/orgs\/[^/]*\/repository-types$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ types: ['service', 'library'] }),
      });
    });

    await page.goto('/create');
    await expect(page.getByText('basic-service')).toBeVisible();
    await page.getByRole('heading', { level: 3, name: 'basic-service' }).click();
    await page.getByRole('button', { name: /next/i }).click();

    await expect(page.getByRole('heading', { name: /repository settings/i })).toBeVisible();
  });

  test('repository name field is visible in Step 2', async ({ page }) => {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: MOCK_TEMPLATES }),
      });
    });
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates\/basic-service$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(MOCK_TEMPLATE_DETAIL_NO_VARS),
      });
    });
    await page.route(/\/api\/v1\/orgs\/[^/]*\/repository-types$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ types: ['service'] }),
      });
    });

    await page.goto('/create');
    await expect(page.getByText('basic-service')).toBeVisible();
    await page.getByRole('heading', { level: 3, name: 'basic-service' }).click();
    await page.getByRole('button', { name: /next/i }).click();

    await expect(page.getByRole('textbox', { name: /repository name/i })).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// SCR-005: Repository Created screen
// ---------------------------------------------------------------------------

test.describe('Repository Created screen (SCR-005)', () => {
  test.beforeEach(async ({ context }) => {
    await addSessionCookie(context);
  });

  test('shows "Repository created!" heading for valid repo param', async ({ page }) => {
    await page.goto('/create/success?repo=test-org/my-repo');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('Repository created!');
  });

  test('shows the full repository name in the page', async ({ page }) => {
    await page.goto('/create/success?repo=test-org/my-repo');
    await expect(page.getByText('test-org/my-repo', { exact: true })).toBeVisible();
  });

  test('"View repository on GitHub" button is present (UX-ASSERT-022)', async ({ page }) => {
    await page.goto('/create/success?repo=test-org/my-repo');
    const btn = page.getByRole('link', { name: 'View repository on GitHub ↗' });
    await expect(btn).toBeVisible();
    await expect(btn).toHaveAttribute('href', 'https://github.com/test-org/my-repo');
  });

  test('"View repository on GitHub" opens in new tab', async ({ page }) => {
    await page.goto('/create/success?repo=test-org/my-repo');
    const btn = page.getByRole('link', { name: 'View repository on GitHub ↗' });
    await expect(btn).toHaveAttribute('target', '_blank');
    await expect(btn).toHaveAttribute('rel', 'noopener noreferrer');
  });

  test('"Create another repository" navigates to /create (UX-ASSERT-023)', async ({ page }) => {
    await page.goto('/create/success?repo=test-org/my-repo');
    const link = page.getByRole('link', { name: 'Create another repository' });
    await expect(link).toHaveAttribute('href', '/create');
  });

  test('invalid/missing repo param shows fallback message', async ({ page }) => {
    await page.goto('/create/success');
    await expect(
      page.getByText('Your repository was created. Check your GitHub organization to find it.'),
    ).toBeVisible();
  });

  test('"Create another repository" shown even with invalid param', async ({ page }) => {
    await page.goto('/create/success');
    await expect(page.getByRole('link', { name: 'Create another repository' })).toBeVisible();
  });
});
