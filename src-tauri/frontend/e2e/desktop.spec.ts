import { test, expect } from '@playwright/test';

test.describe('NeoTrix Desktop App — E2E interaction', () => {
  test('app shell renders with title and status', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/neotrix/i);
  });

  test('NeoCodex sidebar renders session header', async ({ page }) => {
    await page.goto('/');
    const header = page.getByText('NeoCodex', { exact: false }).first();
    await expect(header).toBeVisible({ timeout: 10_000 });
  });

  test('input panel accepts a prompt', async ({ page }) => {
    await page.goto('/');
    const input = page.locator('textarea').first();
    await input.fill('hello neotrix');
    await expect(input).toHaveValue('hello neotrix');
  });

  test('session list area mounts', async ({ page }) => {
    await page.goto('/');
    const list = page.locator('.sidebar, aside');
    await expect(list.first()).toBeVisible({ timeout: 10_000 });
  });

  test('sidebar tabs switch between 会话 and 文件 (标签点击)', async ({ page }) => {
    await page.goto('/');
    const sessionsTab = page.getByTestId('sidebar-tab-sessions');
    const filesTab = page.getByTestId('sidebar-tab-files');
    await expect(sessionsTab).toBeVisible({ timeout: 10_000 });

    // 会话 tab is active by default
    await expect(sessionsTab).toHaveClass(/sidebarTabActive/);

    // click 文件 → active state moves
    await filesTab.click();
    await expect(filesTab).toHaveClass(/sidebarTabActive/);
    await expect(sessionsTab).not.toHaveClass(/sidebarTabActive/);

    // click back
    await sessionsTab.click();
    await expect(sessionsTab).toHaveClass(/sidebarTabActive/);
  });

  test('views menu opens and toggles panels (标签点击)', async ({ page }) => {
    await page.goto('/');
    const viewsBtn = page.getByTestId('views-menu-btn');
    await expect(viewsBtn).toBeVisible({ timeout: 10_000 });

    await viewsBtn.click();
    const terminalItem = page.getByTestId('views-menu-terminal');
    await expect(terminalItem).toBeVisible();

    await terminalItem.click();
    // menu closes after selection
    await expect(terminalItem).not.toBeVisible();
    await expect(viewsBtn).not.toHaveClass(/viewsMenuActive/);
  });

  test('mode selector renders all modes', async ({ page }) => {
    await page.goto('/');
    const mode = page.getByTestId('mode-select');
    await expect(mode).toBeVisible({ timeout: 10_000 });
    await expect(mode).not.toBeDisabled();
    await expect(mode.locator('option')).toHaveCount(3);
    await expect(mode.locator('option')).toContainText(['Agent', 'Shell', 'Plan']);
  });

  test('settings dialog tab switching (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Meta+,').catch(() => {});
    // Fallback: navigate directly
    await page.goto('/settings');
    const providersTab = page.getByTestId('settings-tab-providers');
    const themeTab = page.getByTestId('settings-tab-theme');
    await expect(providersTab).toBeVisible({ timeout: 10_000 });

    await themeTab.click();
    await expect(themeTab).toHaveClass(/active/);

    const advancedTab = page.getByTestId('settings-tab-advanced');
    await advancedTab.click();
    await expect(advancedTab).toHaveClass(/active/);

    const aboutTab = page.getByTestId('settings-tab-about');
    await aboutTab.click();
    await expect(aboutTab).toHaveClass(/active/);
  });

  test('no uncaught errors on load', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text());
    });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    expect(
      errors.filter(
        (e) =>
          !/favicon|tailwind|websocket|tauri|invoke|resizeobserver|transformCallback|__TAURI__|404 \(Not Found\)/i.test(e)
      )
    ).toEqual([]);
  });
});
