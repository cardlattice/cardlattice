import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e', fullyParallel: false, workers: 1, timeout: 45_000,
  use: { baseURL: process.env.WEB_TEST_ORIGIN || 'http://127.0.0.1:5187', trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  webServer: process.env.WEB_TEST_ORIGIN ? undefined : { command: 'pnpm dev --port 5187', url: 'http://127.0.0.1:5187', reuseExistingServer: false },
});
