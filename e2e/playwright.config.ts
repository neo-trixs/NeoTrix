import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: '.',
  timeout: 30_000,
  retries: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  use: {
    headless: true,
  },
  projects: [
    {
      name: 'cli',
      testMatch: '**/cli-smoke.spec.ts',
    },
  ],
});
