const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const COOKIE_FILE = path.join(__dirname, 'xueshu_cookies.json');
const DB_URL = 'https://www.xueshu789.com/dbList/1';

(async () => {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext({
    userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
    locale: 'zh-CN',
  });

  const page = await context.newPage();

  // 尝试加载已有 cookies
  if (fs.existsSync(COOKIE_FILE)) {
    const cookies = JSON.parse(fs.readFileSync(COOKIE_FILE, 'utf8'));
    await context.addCookies(cookies);
    console.log('✅ 已加载已有 cookies');
  }

  await page.goto(DB_URL, { waitUntil: 'networkidle' });
  console.log('🌐 当前 URL:', page.url());

  // 如果被重定向到登录页，等待用户手动登录
  if (page.url().includes('login') || page.url().includes('Login')) {
    console.log('🔑 需要登录，请在浏览器中手动登录...');
    console.log('⏳ 等待登录完成（最长 120 秒）...');
    await page.waitForURL(u => !u.includes('login'), { timeout: 120000 });
    console.log('✅ 登录成功!');
  }

  // 保存 cookies
  const cookies = await context.cookies();
  fs.writeFileSync(COOKIE_FILE, JSON.stringify(cookies, null, 2));
  console.log('💾 cookies 已保存到:', COOKIE_FILE);

  // 抓取文献列表
  const title = await page.title();
  console.log('📄 页面标题:', title);

  // 提取文献条目
  const entries = await page.evaluate(() => {
    const items = [];
    document.querySelectorAll('a[href*="/dbItem/"]').forEach(a => {
      items.push({
        title: a.textContent.trim(),
        url: a.href,
      });
    });
    return items;
  });
  console.log(`📚 找到 ${entries.length} 篇文献`);
  entries.forEach(e => console.log(`  - ${e.title}: ${e.url}`));

  // 保存文献列表
  fs.writeFileSync(path.join(__dirname, 'xueshu_dbList_1.json'), JSON.stringify(entries, null, 2));

  await browser.close();
  console.log('✅ 完成');
})();
