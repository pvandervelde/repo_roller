import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig(({ mode }) => ({
  plugins: [sveltekit()],
  // In test mode, use browser conditions so Svelte resolves to client-side build
  resolve: mode === 'test' ? { conditions: ['browser'] } : {},
  test: {
    include: ['src/**/*.{test,spec}.{js,ts}', 'tests/**/*.{test,spec}.{js,ts}'],
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      // Only track client-side source files.
      // SvelteKit +page.server.ts / +server.ts files run exclusively on the
      // Node.js server side and cannot be exercised in a jsdom environment;
      // they are covered by Playwright E2E tests instead.
      include: ['src/**'],
      exclude: [
        'src/**/*.server.ts',
        'src/**/*.server.js',
        'src/**/+server.ts',
        'src/**/+server.js',
        'src/**/+page.ts',
        'src/**/+layout.ts',
        'src/routes/dev-login/**',
        'src/routes/api/**',
      ],
      thresholds: {
        lines: 80,
        branches: 70,
        functions: 80,
        statements: 80,
      },
    },
  },
}));
