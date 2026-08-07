// 液态玻璃雪域白主题视觉验证
import { chromium } from '@playwright/test';

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });

// Mock Tauri IPC
await page.addInitScript(() => {
  const callbacks = {};
  window.__TAURI_INTERNALS__ = {
    transformCallback: (cb) => { const id = 'cb_' + Object.keys(callbacks).length; callbacks[id] = cb; return id; },
    invoke: async (cmd, args) => {
      if (cmd === 'plugin:event|listen') return 'unlisten-id';
      if (cmd === 'neocodex_list_sessions') return [];
      if (cmd === 'neocodex_provider_config') return { provider_count: 0, resolvable: false, active_model: 'deepseek-v4-flash', providers: [] };
      return [];
    },
  };
});

const errors = [];
page.on('pageerror', err => errors.push('PAGEERROR: ' + err.message.slice(0, 150)));

await page.goto('http://localhost:1447/', { waitUntil: 'networkidle' });
await page.waitForTimeout(2000);

// 1. 空状态
await page.screenshot({ path: '/tmp/neocodex-empty-state.png' });

// 2. 填充消息模拟
await page.evaluate(() => {
  window.__testInject = true;
});
// 通过 localStorage 种子？不行，store 是内存的。直接截图空状态 + 检查计算样式。
const diag = await page.evaluate(() => {
  const body = getComputedStyle(document.body);
  const sidebar = document.querySelector('aside');
  const logo = document.querySelector('.glass-strong, aside h1');
  return {
    bodyBg: body.backgroundImage.slice(0, 60),
    bodyColor: body.color,
    hasSidebar: !!document.querySelector('aside'),
    hasSuggestions: document.querySelectorAll('button.group').length,
    brand: document.querySelector('h1')?.textContent,
    rootLen: document.getElementById('root')?.innerHTML?.length || 0,
  };
});
console.log('DIAG:', JSON.stringify(diag, null, 2));
console.log('ERRORS:', errors.length ? errors.join(' | ') : 'none');

await browser.close();
