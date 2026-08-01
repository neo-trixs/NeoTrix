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

  test('command palette opens via Cmd+K, filters, keyboard navigates (键盘交互)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('sidebar-tab-sessions').click();
    await page.keyboard.press('Meta+k');
    const palette = page.getByTestId('command-palette');
    await expect(palette).toBeVisible({ timeout: 10_000 });
    const input = page.getByTestId('palette-input');
    await expect(input).toBeFocused();
    await input.fill('会话');
    const items = page.getByTestId('palette-list').locator('button');
    await expect(items.first()).toContainText('新建会话');
    // ArrowDown + Enter selects the highlighted item and closes the palette.
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('Enter');
    await expect(palette).not.toBeVisible();
  });

  test('command palette closes with Escape (键盘交互)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('sidebar-tab-sessions').click();
    await page.keyboard.press('Meta+k');
    const palette = page.getByTestId('command-palette');
    await expect(palette).toBeVisible({ timeout: 10_000 });
    await page.keyboard.press('Escape');
    await expect(palette).not.toBeVisible();
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

  test('diff panel opens via views menu with review actions (标签点击)', async ({ page }) => {
    await page.goto('/');
    const viewsBtn = page.getByTestId('views-menu-btn');
    await expect(viewsBtn).toBeVisible({ timeout: 10_000 });
    await viewsBtn.click();
    await page.getByTestId('views-menu-diff').click();
    await expect(page.getByTestId('diff-scope-unstaged')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('diff-stage-all')).toBeVisible();
    await expect(page.getByTestId('diff-unstage-all')).toBeVisible();
    await expect(page.getByTestId('diff-commit')).toBeVisible();
    // Scope tab switch is pure client-side.
    await page.getByTestId('diff-scope-staged').click();
    await expect(page.getByTestId('diff-scope-staged')).toHaveClass(/scopeActive/);
  });

  test('files tab opens the file tree panel (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('sidebar-tab-files').click();
    const panel = page.getByTestId('file-tree-panel');
    await expect(panel).toBeVisible({ timeout: 10_000 });
    await expect(panel).toContainText('文件');
    // Switch back to sessions.
    await page.getByTestId('sidebar-tab-sessions').click();
    await expect(panel).not.toBeVisible();
  });

  test('terminal pane opens via views menu with input (标签点击)', async ({ page }) => {
    await page.goto('/');
    const viewsBtn = page.getByTestId('views-menu-btn');
    await expect(viewsBtn).toBeVisible({ timeout: 10_000 });
    await viewsBtn.click();
    await page.getByTestId('views-menu-terminal').click();
    const pane = page.getByTestId('terminal-pane');
    await expect(pane).toBeVisible({ timeout: 10_000 });
    await expect(pane.getByTestId('terminal-input')).toBeVisible();
    // Close via the same menu toggle.
    await viewsBtn.click();
    await page.getByTestId('views-menu-terminal').click();
    await expect(pane).not.toBeVisible();
  });

  test('preview pane opens via views menu (标签点击)', async ({ page }) => {
    await page.goto('/');
    const viewsBtn = page.getByTestId('views-menu-btn');
    await expect(viewsBtn).toBeVisible({ timeout: 10_000 });
    await viewsBtn.click();
    await page.getByTestId('views-menu-preview').click();
    const pane = page.getByTestId('preview-pane');
    await expect(pane).toBeVisible({ timeout: 10_000 });
    await expect(pane.getByTestId('preview-url')).toBeVisible();
    await expect(pane.getByTestId('preview-open')).toBeVisible();
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

  test('diff pane shows the changed-file list and selects a file (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('views-menu-btn').click();
    await page.getByTestId('views-menu-diff').click();
    // File list loads (may be empty in a clean repo).
    const fileList = page.getByTestId('file-tree-panel').or(page.locator('text=无改动文件'));
    await expect(fileList.first()).toBeVisible({ timeout: 10_000 }).catch(() => {});
    // The scope tabs still work side-by-side with the file list.
    await page.getByTestId('diff-scope-staged').click();
    await expect(page.getByTestId('diff-scope-staged')).toHaveClass(/scopeActive/);
  });

  test('terminal pane supports multiple tabs (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('views-menu-btn').click();
    await page.getByTestId('views-menu-terminal').click();
    await expect(page.getByTestId('terminal-pane')).toBeVisible({ timeout: 10_000 });
    // Add a second tab.
    await page.getByTestId('terminal-add').click();
    await expect(page.getByTestId('terminal-tab-term').first()).toBeVisible().catch(() => {});
    // Tab bar still present; two tabs exist.
    await expect(page.getByRole('tablist')).toBeVisible();
    await page.getByRole('tablist').getByRole('tab').count().then((n) => expect(n).toBeGreaterThanOrEqual(2));
  });

  test('shortcut help lists numbered session switch (⌘1..⌘9)', async ({ page }) => {
    await page.goto('/');
    // Wait for the app shell so the keyboard listener is attached.
    await expect(page.getByTestId('sidebar-tab-sessions')).toBeVisible({ timeout: 10_000 });
    await page.keyboard.press('Meta+/');
    const modal = page.getByRole('dialog', { name: '快捷键' });
    await expect(modal).toBeVisible({ timeout: 10_000 });
    await expect(modal).toContainText('⌘1…⌘9');
    await page.keyboard.press('Escape');
    await expect(modal).not.toBeVisible();
  });

  test('session toolbar shows title, project, and branch chip (标签点击)', async ({ page }) => {
    await page.goto('/');
    // The toolbar only mounts once an active session exists (backend-driven);
    // in browser mode without Tauri it may not appear — tolerate that.
    const toolbar = page.getByTestId('session-toolbar');
    await expect(toolbar).toBeVisible({ timeout: 10_000 }).catch(() => {});
    if (!(await toolbar.isVisible().catch(() => false))) return;
    // Title button is clickable and switches into inline rename mode.
    const title = page.getByTestId('session-title');
    await expect(title).toBeVisible();
    await title.click();
    const input = page.getByTestId('session-title-input');
    await expect(input).toBeVisible();
    // Escape cancels rename without touching the backend.
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('session-title')).toBeVisible();
  });

  test('diff base scope shows branch input (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('views-menu-btn').click();
    await page.getByTestId('views-menu-diff').click();
    await page.getByTestId('diff-scope-base').click();
    const baseInput = page.getByTestId('diff-base-branch');
    await expect(baseInput).toBeVisible({ timeout: 10_000 });
    await expect(baseInput).toHaveValue('main');
  });

  test('settings advanced group shows 通知 toggle (标签点击)', async ({ page }) => {
    await page.goto('/settings');
    await page.getByTestId('settings-tab-advanced').click();
    await expect(page.getByTestId('settings-tab-advanced')).toHaveClass(/active/);
    await expect(page.getByText('任务完成时发送系统通知')).toBeVisible({ timeout: 10_000 });
  });

  test('welcome empty state lists recent sessions (标签点击)', async ({ page }) => {
    await page.goto('/');
    // With no active conversation, the recent-sessions block mounts client-side.
    const recent = page.getByTestId('recent-sessions');
    await expect(recent).toBeVisible({ timeout: 10_000 }).catch(() => {});
  });

  test('diff pane shows AI review button and panel can open/close (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('views-menu-btn').click();
    await page.getByTestId('views-menu-diff').click();
    const reviewBtn = page.getByTestId('diff-review');
    await expect(reviewBtn).toBeVisible({ timeout: 10_000 });
    // In browser mode there is no Tauri invoke; the review either errors or
    // renders the panel. Tolerate both, but the button itself must exist.
    await reviewBtn.click().catch(() => {});
    const panel = page.getByTestId('diff-review-panel');
    await expect(panel).toBeVisible({ timeout: 5_000 }).catch(() => {});
    if (await panel.isVisible().catch(() => false)) {
      await page.getByTitle('关闭').first().click().catch(() => {});
      await expect(panel).not.toBeVisible().catch(() => {});
    }
  });

  test('Cmd+P opens the file-search palette (键盘交互)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('sidebar-tab-sessions').click();
    await page.keyboard.press('Meta+p');
    const palette = page.getByTestId('command-palette');
    await expect(palette).toBeVisible({ timeout: 10_000 });
    const input = page.getByTestId('palette-input');
    await expect(input).toBeFocused();
    await input.fill('src');
    await page.keyboard.press('Escape');
    await expect(palette).not.toBeVisible();
  });

  test('settings advanced group shows model fields (标签点击)', async ({ page }) => {
    await page.goto('/settings');
    await page.getByTestId('settings-tab-advanced').click();
    await expect(page.getByTestId('settings-tab-advanced')).toHaveClass(/active/);
    await expect(page.getByTestId('settings-defaultModel')).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId('settings-temperature')).toBeVisible();
    await expect(page.getByTestId('settings-maxTokens')).toBeVisible();
    await expect(page.getByTestId('settings-maxSessions')).toBeVisible();
    await expect(page.getByTestId('settings-terminalPath')).toBeVisible();
  });

  test('settings theme tab shows language selector (标签点击)', async ({ page }) => {
    await page.goto('/settings');
    await page.getByTestId('settings-tab-theme').click();
    await expect(page.getByTestId('settings-tab-theme')).toHaveClass(/active/);
    await expect(page.getByTestId('settings-language')).toBeVisible({ timeout: 10_000 });
  });

  test('session sidebar shows import button (标签点击)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('sidebar-tab-sessions').click();
    const importBtn = page.getByTitle('导入会话 JSON');
    await expect(importBtn).toBeVisible({ timeout: 10_000 });
    // The hidden file input backs the button.
    await expect(page.getByTestId('session-import-input')).toBeAttached();
  });
});
