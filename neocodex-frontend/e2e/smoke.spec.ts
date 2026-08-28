import { test, expect } from '@playwright/test'

/* ════════════════════════════════════════════
   e2e/smoke.spec.ts — 发布前冒烟骨架
   注意：选择器优先用 role/text（对齐 SettingsModal 的 aria 结构），
   Tauri 真机需用 webDriverUrl 指向 tauri-driver 并改用 data-testid。
   ════════════════════════════════════════════ */

test.describe('NeoTrix 冒烟', () => {
  test('应用启动后可渲染主界面', async ({ page }) => {
    await page.goto('/')
    // 核心区域可达：聊天输入区或侧栏其一存在
    const chatReady = page.getByPlaceholder(/输入|消息|说点什么/i).first()
    await expect(chatReady).toBeVisible({ timeout: 15_000 })
  })

  test('设置面板可打开并展示分类', async ({ page }) => {
    await page.goto('/')
    // 打开设置（齿轮/命令）。优先快捷键 Cmd/Ctrl+, 兜底点击齿轮按钮
    const settingsBtn = page.getByRole('button', { name: /设置|settings/i }).first()
    if (await settingsBtn.isVisible().catch(() => false)) {
      await settingsBtn.click()
    } else {
      await page.keyboard.press(process.platform === 'darwin' ? 'Meta+,' : 'Control+,')
    }
    const dialog = page.getByRole('dialog', { name: /设置/i })
    await expect(dialog).toBeVisible({ timeout: 10_000 })
    // 分类导航存在：通用 / 模型 / 外观
    await expect(dialog.getByRole('tab', { name: /通用|模型|外观/i }).first()).toBeVisible()
  })

  test('模型标签可切换并展示代理池', async ({ page }) => {
    await page.goto('/')
    const settingsBtn = page.getByRole('button', { name: /设置|settings/i }).first()
    if (await settingsBtn.isVisible().catch(() => false)) {
      await settingsBtn.click()
    } else {
      await page.keyboard.press(process.platform === 'darwin' ? 'Meta+,' : 'Control+,')
    }
    const dialog = page.getByRole('dialog', { name: /设置/i })
    await dialog.getByRole('tab', { name: /模型/i }).click()
    // 代理池数字徽章（N 提供商）存在
    await expect(dialog.getByText(/提供商/)).toBeVisible({ timeout: 10_000 })
  })
})
