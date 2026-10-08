// neobot-ui 熔炼落点实测 v2 —— mock Tauri IPC，验证：
//  1) 未读角标（D组 unread 落点）
//  2) 草稿按会话分区：A 打字 → 切 B 不串台 → 切回 A 恢复
const { chromium } = require('playwright');
const fs = require('fs');

const CONVOS = [
  { id: 'c1', kind: 'dm', title: '熔炼验证会话 A', members: ['me'], task_count: 2, last_active: '2026-10-08 11:50:00', muted: false, unread: 3 },
  { id: 'c2', kind: 'dm', title: '会话 B', members: ['me'], task_count: 0, last_active: '2026-10-08 11:40:00', muted: false, unread: 0 },
];
const MSGS = {
  c1: [{ id: 'm1', convo_id: 'c1', role: 'assistant', text: '会话 A 的历史消息：草稿分区测试基线。', created_at: '2026-10-08 11:50:00' }],
  c2: [{ id: 'm2', convo_id: 'c2', role: 'assistant', text: '会话 B 的历史消息。', created_at: '2026-10-08 11:40:00' }],
};

const INIT = `(() => {
  const cbs = {}; let seq = 0;
  window.__TAURI_INTERNALS__ = {
    transformCallback: (cb) => { const id = ++seq; cbs[id] = cb; return id; },
    metadata: () => ({ currentWindow: { label: 'main', identifier: 'main', appPrivateInfo: {} }, currentWebview: { label: 'main' } }),
    invoke: (cmd, args) => {
      switch (cmd) {
        case 'neobot_convo_list': return Promise.resolve(${JSON.stringify(CONVOS)});
        case 'neobot_convo_messages_page': return Promise.resolve({ messages: ${JSON.stringify(MSGS)}[args.convoId] || [], hasMore: false, nextSeq: null });
        case 'neobot_usage_summary': return Promise.resolve({ days: 1, input: 0, cached: 0, written: 0, output: 0, requests: 0, tokens: 0 });
        case 'neobot_core_capabilities': return Promise.resolve({ crystal_version: 'mock', tool_count: 0, model: 'mock-model', model_source: 'mock' });
        case 'neobot_member_list': return Promise.resolve([{ id: 'me', kind: 'human', display: 'Me' }]);
        case 'neobot_memory_list': return Promise.resolve({ lines: [], bytes: 0, cap: 0, revisions: 0 });
        case 'neobot_run_list': return Promise.resolve({ runs: [] });
        default: return Promise.resolve(null);
      }
    },
  };
})();`;

(async () => {
  const exe = process.env.HOME + '/Library/Caches/ms-playwright/chromium-1234/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing';
  const browser = await chromium.launch({ executablePath: exe });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = [];
  page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  await page.addInitScript(INIT);
  await page.goto('http://localhost:1433/', { waitUntil: 'networkidle', timeout: 30000 });

  const result = {};
  // 1. 会话列表 + 未读角标（行可访问名内含未读数）
  const rowA = page.getByRole('button', { name: /熔炼验证会话 A/ });
  const rowB = page.getByRole('button', { name: /会话 B/ });
  await rowA.waitFor({ timeout: 10000 });
  result.rowAName = await rowA.getAttribute('aria-label').catch(() => null);
  result.unreadVisible = /3/.test(await rowA.innerText().catch(() => ''));
  await page.screenshot({ path: '/tmp/neobot-ui-shots/03-convos-unread.png' });

  // 2. 选 A → 打草稿
  await rowA.click();
  await page.waitForTimeout(600);
  const ta = page.locator('textarea').first();
  await ta.click();
  await ta.fill('草稿A：分区防串台 123');
  await page.waitForTimeout(400);
  result.draftInA = await ta.inputValue();
  await page.screenshot({ path: '/tmp/neobot-ui-shots/04-draft-in-A.png' });

  // 3. 切 B → 草稿不得串台（应为空）
  await rowB.click();
  await page.waitForTimeout(600);
  result.draftInB_afterSwitch = await ta.inputValue();
  await page.screenshot({ path: '/tmp/neobot-ui-shots/05-draft-not-leak-B.png' });

  // 4. 切回 A → 草稿应恢复
  await rowA.click();
  await page.waitForTimeout(600);
  result.draftBackToA = await ta.inputValue();

  result.pageErrors = errors.slice(0, 5);
  result.pass = result.unreadVisible && result.draftInA.includes('草稿A') && result.draftInB_afterSwitch === '' && result.draftBackToA.includes('草稿A');
  fs.writeFileSync('/tmp/neobot-ui-shots/draft-isolation.json', JSON.stringify(result, null, 2));
  console.log('RESULT=' + JSON.stringify(result));
  await browser.close();
  process.exit(result.pass ? 0 : 2);
})().catch((e) => { console.error('FATAL=' + e.message); process.exit(1); });
