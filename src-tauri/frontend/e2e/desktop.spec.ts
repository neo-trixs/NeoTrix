import { test, expect } from '@playwright/test';

test.describe('NeoTrix Desktop App — E2E smoke', () => {
  test('app shell renders with title and status', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/neotrix/i);
  });

  test('root layout exposes the StatusBar', async ({ page }) => {
    await page.goto('/');
    const status = page.getByTestId('status-bar').or(page.locator('[data-testid="status-bar"]'));
    await expect(status).toBeVisible({ timeout: 10_000 });
  });

  test('input panel accepts a prompt', async ({ page }) => {
    await page.goto('/');
    const input = page.locator('textarea, input[type="text"]').first();
    await input.fill('hello neotrix');
    await expect(input).toHaveValue('hello neotrix');
  });

  test('session list area mounts', async ({ page }) => {
    await page.goto('/');
    const list = page.getByTestId('session-list').or(page.locator('[data-testid="session-list"]'));
    await expect(list).toBeVisible({ timeout: 10_000 });
  });

  test('no uncaught errors on load', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text());
    });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    expect(errors.filter((e) => !/favicon|tailwind|websocket|tauri/i.test(e))).toEqual([]);
  });
});
