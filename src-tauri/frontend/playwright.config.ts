import { defineConfig, devices } from '@playwright/test'

/* ════════════════════════════════════════════
   playwright.config.ts — E2E 冒烟测试骨架
   对标 dsh-desktop / Claude Desktop 的发布前冒烟：
   启动桌面应用（或 vite preview）后验证核心 UI 可达。
   真机 Tauri 场景用 webDriverUrl 指向 tauri-driver；
   此处默认对 vite preview 跑无头冒烟，CI 可覆盖 baseURL。
   ════════════════════════════════════════════ */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'list',
  use: {
    baseURL: process.env.E2E_BASE_URL ?? 'http://localhost:4173',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: process.env.E2E_BASE_URL
    ? undefined
    : {
        command: 'npm run preview -- --port 4173',
        url: 'http://localhost:4173',
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
      },
})
