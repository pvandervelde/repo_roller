import { test, expect } from '@playwright/test';

test('sign-in page renders with a heading', async ({ page }) => {
  await page.goto('/sign-in');
  await expect(page).toHaveTitle(/Sign in/);
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
});

test('unauthenticated request to /create redirects to /sign-in', async ({ page }) => {
  await page.goto('/create');
  await expect(page).toHaveURL('/sign-in');
});

test('unauthenticated request to /create/success redirects to /sign-in', async ({ page }) => {
  await page.goto('/create/success');
  await expect(page).toHaveURL('/sign-in');
});
