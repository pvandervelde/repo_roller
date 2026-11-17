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
    displayName: 'Basic Service',
    description: 'A basic service template',
    tags: ['service'],
    repository_type_name: 'service',
    repository_type_policy: 'preferable',
    variable_count: 0,
  },
  {
    name: 'data-pipeline',
    displayName: 'Data Pipeline',
    description: 'A data pipeline template',
    tags: ['data'],
    repository_type_name: null,
    repository_type_policy: 'optional',
    variable_count: 2,
  },
];

const MOCK_TEMPLATE_DETAIL_NO_VARS = {
  name: 'basic-service',
  displayName: 'Basic Service',
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
    await expect(page.getByRole('heading', { level: 3, name: 'Basic Service' })).toBeVisible();
    await expect(page.getByRole('heading', { level: 3, name: 'Data Pipeline' })).toBeVisible();
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
    await expect(page.getByRole('heading', { level: 3, name: 'Basic Service' })).toBeVisible();
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
    await expect(page.getByRole('heading', { level: 3, name: 'Basic Service' })).toBeVisible();
    await page.getByRole('heading', { level: 3, name: 'Basic Service' }).click();
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
        body: JSON.stringify({
          types: [
            { name: 'service', description: 'Backend service or REST API' },
            { name: 'library', description: 'Reusable shared library' },
          ],
        }),
      });
    });

    await page.goto('/create');
    await expect(page.getByRole('heading', { level: 3, name: 'Basic Service' })).toBeVisible();
    await page.getByRole('heading', { level: 3, name: 'Basic Service' }).click();
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
        body: JSON.stringify({
          types: [{ name: 'service', description: 'Backend service or REST API' }],
        }),
      });
    });

    await page.goto('/create');
    await expect(page.getByRole('heading', { level: 3, name: 'Basic Service' })).toBeVisible();
    await page.getByRole('heading', { level: 3, name: 'Basic Service' }).click();
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

// ---------------------------------------------------------------------------
// Fork and Mirror creation (WUI-005, WUIE-003) — Task #2
//
// RED-phase note: these scenarios assume the same mode-radiogroup / source
// -repository-textbox / history-preservation-radiogroup contract documented
// in frontend/tests/CreateWizard.test.ts's "Task #2" section. Keep both in
// sync if the Coder's final copy/structure differs from these assumptions.
// ---------------------------------------------------------------------------

test.describe('Creation wizard — Fork and Mirror creation (WUI-005)', () => {
  test.beforeEach(async ({ context }) => {
    await addSessionCookie(context);
  });

  async function mockEmptyTemplatesRoute(page: import('@playwright/test').Page) {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/templates$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ templates: [] }),
      });
    });
  }

  async function mockCapabilities(
    page: import('@playwright/test').Page,
    overrides: Record<string, unknown>,
  ) {
    await page.route(/\/api\/v1\/orgs\/[^/]*\/creation-capabilities.*$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          forkEnabled: false,
          sourceHistoryMirrorEnabled: false,
          sourceOnlyMirrorEnabled: false,
          allowedSourceOwners: [],
          forkTemplate: null,
          sourceHistoryMirrorTemplate: null,
          sourceOnlyMirrorTemplate: null,
          ...overrides,
        }),
      });
    });
  }

  // RepositoryNameField's availability check only fires on blur, and the
  // Settings step's "Next: Review" button stays disabled until that check
  // resolves to valid+available. Every scenario below that reaches Review
  // must mock this endpoint AND explicitly blur the name field after
  // filling it -- `.fill()` alone never blurs, and a genuinely-disabled
  // button cannot receive focus in real Chromium (unlike the jsdom unit
  // tests, where `fireEvent.click` bypasses native disabled-button
  // suppression).
  async function mockValidateNameRoute(page: import('@playwright/test').Page) {
    await page.route(/\/api\/v1\/repositories\/validate-name$/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ valid: true, available: true }),
      });
    });
  }

  /** Fills the repository-name field and blurs it so the async availability
   * check (mocked via `mockValidateNameRoute`) resolves and "Next: Review"
   * becomes enabled. */
  async function fillAndBlurRepositoryName(page: import('@playwright/test').Page, name: string) {
    const nameField = page.getByRole('textbox', { name: /repository name/i });
    await nameField.fill(name);
    await nameField.blur();
    await expect(page.getByRole('button', { name: /next: review/i })).toBeEnabled();
  }

  test('successful fork creation end-to-end sends the exact fork creationMode payload', async ({
    page,
  }) => {
    await mockEmptyTemplatesRoute(page);
    await mockCapabilities(page, { forkEnabled: true, forkTemplate: 'fork-settings' });
    await mockValidateNameRoute(page);

    let capturedBody: Record<string, unknown> | null = null;
    await page.route(/\/api\/v1\/repositories$/, async (route) => {
      capturedBody = route.request().postDataJSON();
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          repository: {
            name: 'my-fork',
            fullName: 'test-org/my-fork',
            url: 'https://github.com/test-org/my-fork',
            visibility: 'private',
          },
          appliedConfiguration: {},
          createdAt: '2026-01-01T00:00:00Z',
        }),
      });
    });

    await page.goto('/create');
    await page.getByRole('radio', { name: 'Fork' }).click();
    await page.getByRole('textbox', { name: /source repository/i }).fill('upstream-org/some-repo');
    await page.getByRole('button', { name: /next/i }).click();
    await fillAndBlurRepositoryName(page, 'my-fork');
    await page.getByRole('button', { name: /next: review/i }).click();
    await page.getByRole('button', { name: /create repository/i }).click();

    await expect(page).toHaveURL(/\/create\/success\?repo=/);
    expect(capturedBody?.creationMode).toEqual({
      type: 'fork',
      source: { owner: 'upstream-org', name: 'some-repo' },
    });
    expect(capturedBody).not.toHaveProperty('template');
  });

  test('successful mirror (preserve history) creation end-to-end sends preserveHistory:true', async ({
    page,
  }) => {
    await mockEmptyTemplatesRoute(page);
    await mockCapabilities(page, {
      sourceHistoryMirrorEnabled: true,
      sourceOnlyMirrorEnabled: true,
    });
    await mockValidateNameRoute(page);

    let capturedBody: Record<string, unknown> | null = null;
    await page.route(/\/api\/v1\/repositories$/, async (route) => {
      capturedBody = route.request().postDataJSON();
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          repository: {
            name: 'my-mirror',
            fullName: 'test-org/my-mirror',
            url: 'https://github.com/test-org/my-mirror',
            visibility: 'private',
          },
          appliedConfiguration: {},
          createdAt: '2026-01-01T00:00:00Z',
        }),
      });
    });

    await page.goto('/create');
    await page.getByRole('radio', { name: 'Mirror' }).click();
    await page.getByRole('textbox', { name: /source repository/i }).fill('upstream-org/some-repo');
    await page.getByRole('radio', { name: /preserve.*history/i }).click();
    await page.getByRole('button', { name: /next/i }).click();
    await fillAndBlurRepositoryName(page, 'my-mirror');
    await page.getByRole('button', { name: /next: review/i }).click();
    await page.getByRole('button', { name: /create repository/i }).click();

    await expect(page).toHaveURL(/\/create\/success\?repo=/);
    expect(capturedBody?.creationMode).toMatchObject({ type: 'mirror', preserveHistory: true });
  });

  test('successful mirror (source-only, no toggle shown) creation end-to-end sends preserveHistory:false', async ({
    page,
  }) => {
    await mockEmptyTemplatesRoute(page);
    await mockCapabilities(page, { sourceOnlyMirrorEnabled: true });
    await mockValidateNameRoute(page);

    let capturedBody: Record<string, unknown> | null = null;
    await page.route(/\/api\/v1\/repositories$/, async (route) => {
      capturedBody = route.request().postDataJSON();
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          repository: {
            name: 'my-mirror2',
            fullName: 'test-org/my-mirror2',
            url: 'https://github.com/test-org/my-mirror2',
            visibility: 'private',
          },
          appliedConfiguration: {},
          createdAt: '2026-01-01T00:00:00Z',
        }),
      });
    });

    await page.goto('/create');
    await page.getByRole('radio', { name: 'Mirror' }).click();
    await page.getByRole('textbox', { name: /source repository/i }).fill('upstream-org/some-repo');
    // Only one mirror sub-type is enabled -- no history-preservation toggle.
    await expect(page.getByRole('radiogroup', { name: /history preservation/i })).toHaveCount(0);
    await page.getByRole('button', { name: /next/i }).click();
    await fillAndBlurRepositoryName(page, 'my-mirror2');
    await page.getByRole('button', { name: /next: review/i }).click();
    await page.getByRole('button', { name: /create repository/i }).click();

    await expect(page).toHaveURL(/\/create\/success\?repo=/);
    expect(capturedBody?.creationMode).toMatchObject({ type: 'mirror', preserveHistory: false });
  });

  test('CapabilityDisabled rejection shows a specific inline message and preserves the source field (WUIE-003)', async ({
    page,
  }) => {
    await mockEmptyTemplatesRoute(page);
    await mockCapabilities(page, { forkEnabled: true });
    await mockValidateNameRoute(page);

    await page.route(/\/api\/v1\/repositories$/, async (route) => {
      await route.fulfill({
        status: 403,
        contentType: 'application/json',
        body: JSON.stringify({
          error: {
            code: 'CapabilityDisabled',
            message: "Capability 'fork' is disabled by configuration",
          },
        }),
      });
    });

    await page.goto('/create');
    await page.getByRole('radio', { name: 'Fork' }).click();
    await page.getByRole('textbox', { name: /source repository/i }).fill('upstream-org/some-repo');
    await page.getByRole('button', { name: /next/i }).click();
    await fillAndBlurRepositoryName(page, 'my-fork');
    await page.getByRole('button', { name: /next: review/i }).click();
    await page.getByRole('button', { name: /create repository/i }).click();

    await expect(page.getByRole('alert')).toBeVisible();
    await expect(page.getByRole('alert')).toContainText(/disabl/i);

    // Review -> Settings -> Source: the source reference typed earlier must survive.
    await page.getByRole('button', { name: /← back/i }).click();
    await page.getByRole('button', { name: /← back/i }).click();
    await expect(page.getByRole('textbox', { name: /source repository/i })).toHaveValue(
      'upstream-org/some-repo',
    );
  });
});
