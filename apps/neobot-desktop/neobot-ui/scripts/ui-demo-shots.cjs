// neobot-ui 熔炼落点实测截图脚本（只读，不写任何项目文件）
const { chromium } = require('playwright');
const fs = require('fs');

(async () => {
  const exe = process.env.HOME + '/Library/Caches/ms-playwright/chromium-1234/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing';
  const browser = await chromium.launch({ executablePath: exe });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = [];
  page.on('pageerror', (e) => errors.push('pageerror: ' + e.message));
  page.on('console', (m) => { if (m.type() === 'error') errors.push('console: ' + m.text()); });

  await page.goto('http://localhost:1433/', { waitUntil: 'networkidle', timeout: 30000 });
  await page.waitForTimeout(1500);

  // 1. 全页首图
  await page.screenshot({ path: '/tmp/neobot-ui-shots/01-root.png', fullPage: false });

  // 2. 结构盘点：侧栏/会话列表/输入框/顶栏存在性
  const probe = await page.evaluate(() => {
    const q = (sel) => !!document.querySelector(sel);
    const count = (sel) => document.querySelectorAll(sel).length;
    return {
      title: document.title,
      bodyTextHead: (document.body.innerText || '').slice(0, 400),
      hasTextarea: count('textarea') > 0,
      textareaCount: count('textarea'),
      hasButton: count('button'),
      unreadBadges: count('[class*="unread"], [data-state*="unread"]'),
      detailsElems: count('details'),
      lang: document.documentElement.lang,
    };
  });
  fs.writeFileSync('/tmp/neobot-ui-shots/probe.json', JSON.stringify(probe, null, 2));

  // 3. 若有 textarea：注入草稿 → 切走再切回验证草稿保留（D组「停止保留草稿」近似）
  if (probe.hasTextarea) {
    const ta = page.locator('textarea').first();
    await ta.click();
    await ta.fill('熔炼验证草稿：按会话分区不应串台');
    await page.screenshot({ path: '/tmp/neobot-ui-shots/02-draft-typed.png' });
    const v1 = await ta.inputValue();
    fs.writeFileSync('/tmp/neobot-ui-shots/draft-check.txt', `typed_len=${v1.length} kept=${v1.includes('熔炼验证草稿')}`);
  }

  console.log('PROBE=' + JSON.stringify(probe));
  console.log('ERRORS=' + JSON.stringify(errors.slice(0, 10)));
  await browser.close();
})().catch((e) => { console.error('FATAL=' + e.message); process.exit(1); });
