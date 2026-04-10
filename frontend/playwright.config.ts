import { defineConfig, devices } from '@playwright/test';
import { E2E_SESSION_SECRET } from './e2e/helpers';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'github' : 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'pnpm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    // Environment variables required for the SvelteKit dev server.
    // SESSION_SECRET must match E2E_SESSION_SECRET so that cookies signed
    // in tests are accepted by the server's parseSessionCookie().
    // GITHUB_* values are placeholders; E2E tests mock all API calls.
    env: {
      SESSION_SECRET: E2E_SESSION_SECRET,
      GITHUB_CLIENT_ID: 'e2e-placeholder-client-id',
      GITHUB_CLIENT_SECRET: 'e2e-placeholder-client-secret',
      GITHUB_ORG: 'e2e-test-org',
    },
  },
});
