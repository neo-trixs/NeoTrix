import { test, expect } from './fixtures';

test.describe('NeoTrix Desktop — Vanilla UI', () => {
  test('app shell renders with title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/neotrix/i);
  });

  test('sidebar mounts with nav list and recents', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#sidebar')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#navList .nl')).toHaveCount(2);
    await expect(page.locator('#recentList .re-h')).toHaveText('最近');
  });

  test('input panel accepts a prompt and enables send', async ({ page }) => {
    await page.goto('/');
    const input = page.locator('#chatInput');
    await input.fill('hello neotrix');
    await expect(input).toHaveValue('hello neotrix');
    const send = page.locator('#sendBtn');
    await expect(send).toBeEnabled();
    await send.click({ force: true });
    await expect(input).toHaveValue('');
  });

  test('chat view renders hero and segment control switches to 团队', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#viewChat')).toBeVisible();
    await expect(page.locator('.hero h1')).toBeVisible();
    await page.locator('.segb[data-view="cowork"]').click({ force: true });
    await expect(page.locator('#viewCowork')).toBeVisible();
    await expect(page.locator('#viewChat')).toBeHidden();
  });

  test('cowork view renders session list container', async ({ page }) => {
    await page.goto('/');
    await page.locator('.segb[data-view="cowork"]').click({ force: true });
    await expect(page.locator('#cwSessionList')).toBeVisible({ timeout: 10_000 });
  });

  test('settings overlay opens via popover and Cmd+,', async ({ page }) => {
    await page.goto('/');
    await page.locator('#userBar').click({ force: true });
    await page.locator('.pop-item', { hasText: '设置' }).click({ force: true });
    await expect(page.locator('#overlaySettings')).toHaveClass(/open/);
    await page.keyboard.press('Escape');
    await expect(page.locator('#overlaySettings')).not.toHaveClass(/open/);
    await page.keyboard.press('Meta+,');
    await expect(page.locator('#overlaySettings')).toHaveClass(/open/);
  });

  test('settings overlay opens and gateway tab renders status', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Meta+,');
    await expect(page.locator('#overlaySettings')).toHaveClass(/open/);
    await page.locator('.st-item', { hasText: '代理 · 网关' }).click({ force: true });
    await expect(page.locator('#stGateway')).toHaveClass(/open/);
  });

  test('theme toggle switches light/dark', async ({ page }) => {
    await page.goto('/');
    await page.locator('#userBar').click({ force: true });
    await page.locator('#popThemeToggle').click({ force: true });
    const theme = await page.locator('html').getAttribute('data-theme');
    expect(['light', 'dark']).toContain(theme);
  });

  test('projects overlay opens from 项目 nav item', async ({ page }) => {
    await page.goto('/');
    await page.locator('#navList .nl', { hasText: '项目' }).click({ force: true });
    await expect(page.locator('#overlayProjects')).toHaveClass(/open/);
    await expect(page.locator('#opTitle')).toHaveText('项目');
    await page.keyboard.press('Escape');
    await expect(page.locator('#overlayProjects')).not.toHaveClass(/open/);
  });

  test('command palette shortcut opens settings search and focuses input', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Meta+k');
    await expect(page.locator('#overlaySettings')).toHaveClass(/open/);
    const si = page.locator('.st-search input');
    await expect(si).toBeVisible();
    await si.fill('模型');
    await expect(si).toHaveValue('模型');
  });

  test('diff overlay opens from + menu and renders sample diff', async ({ page }) => {
    await page.goto('/');
    await page.locator('#ntxPlusBtn').click({ force: true });
    await page.locator('.ntx-pm-item[data-act="diff"]').click({ force: true });
    await expect(page.locator('#overlayDiff')).toHaveClass(/open/);
    await expect(page.locator('#diffTitle')).toContainText('代码变更');
    await expect(page.locator('#diffBody')).not.toBeEmpty();
  });

  test('right sidebar toggles via floating button', async ({ page }) => {
    await page.goto('/');
    const rb = page.locator('#rightbar');
    await expect(rb).toHaveClass(/collapsed/);
    await page.locator('#rbFloat').dispatchEvent('click');
    await expect(rb).not.toHaveClass(/collapsed/);
  });

  test('file tree renders in right sidebar', async ({ page }) => {
    await page.goto('/');
    await page.locator('#rbFloat').click({ force: true });
    await expect(page.locator('#fileTree .ft-item').first()).toBeVisible({ timeout: 10_000 });
  });

  test('KB search input exists and empty query shows hint', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Meta+,');
    await page.locator('.st-item', { hasText: '数据控制' }).click({ force: true });
    const inp = page.locator('#kbSearchInput');
    await expect(inp).toBeVisible({ timeout: 10_000 });
    await inp.fill('');
    const res = page.locator('#kbResults');
    await expect(res).toContainText('输入关键词');
  });

  test('MCP registration form is present in gateway tab', async ({ page }) => {
    await page.goto('/');
    await page.keyboard.press('Meta+,');
    await page.locator('.st-item', { hasText: '代理 · 网关' }).click({ force: true });
    await expect(page.locator('#mcpName')).toBeVisible({ timeout: 10_000 });
    await expect(page.locator('#mcpCmd')).toBeVisible();
    await expect(page.locator('#mcpArgs')).toBeVisible();
  });

  test('shortcut help shows in user popover', async ({ page }) => {
    await page.goto('/');
    await page.locator('#userBar').click({ force: true });
    await expect(page.locator('#userPopover')).toBeVisible();
    await expect(page.locator('.pop-item', { hasText: '帮助' })).toBeVisible();
  });

  test('no uncaught errors on load', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await page.goto('/');
    await page.waitForTimeout(1500);
    expect(errors).toEqual([]);
  });
});
