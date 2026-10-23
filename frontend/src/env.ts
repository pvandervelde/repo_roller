/**
 * Environment variable declarations for @sveltejs/kit 3's `$app/env/private`
 * module (replaces the old `$env/dynamic/private`).
 *
 * All variables here are private (server-only) and dynamic (read from the
 * real environment when the app starts, never inlined at build time) —
 * matching the behaviour of the old `$env/dynamic/private`. Each has a
 * passthrough schema so a missing value resolves to `undefined` instead of
 * throwing at startup, since none of these are required for the app to boot
 * (dev mode and the mock API routes run without them).
 */
import { defineEnvVars } from '@sveltejs/kit/env';

const passthrough = (value: string | undefined) => value;

export const variables = defineEnvVars({
  BACKEND_API_URL: {
    schema: passthrough,
    description:
      'Base URL of the Rust API backend, e.g. http://api:3000. When unset, API routes return mock data instead of proxying.',
  },
  GITHUB_CLIENT_ID: {
    schema: passthrough,
    description: 'GitHub OAuth App client ID, required for sign-in.',
  },
  GITHUB_CLIENT_SECRET: {
    schema: passthrough,
    description: 'GitHub OAuth App client secret, required for sign-in.',
  },
  NODE_ENV: {
    schema: passthrough,
    description: "Node environment; used to toggle cookies' secure flag.",
  },
});
