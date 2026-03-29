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
            include: ['src/**'],
        },
    },
}));
