#!/usr/bin/env python3
"""DeepSeek 网页扫码登录：可见浏览器 + 持久 profile，轮询登录态。"""
import sys
import time
from playwright.sync_api import sync_playwright

PROFILE = "datasets/deepseek/profile"
TIMEOUT = int(sys.argv[1]) if len(sys.argv) > 1 else 300

with sync_playwright() as p:
    ctx = p.chromium.launch_persistent_context(
        PROFILE, channel="chrome", headless=False,
        args=["--disable-blink-features=AutomationControlled"],
        user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                   "AppleWebKit/537.36 (KHTML, like Gecko) "
                   "Chrome/126.0.0.0 Safari/537.36",
        viewport={"width": 1400, "height": 900})
    pg = ctx.pages[0] if ctx.pages else ctx.new_page()
    pg.goto("https://chat.deepseek.com/", wait_until="domcontentloaded",
            timeout=60000)
    print("请扫码登录 DeepSeek（等待%d秒）…" % TIMEOUT)
    t0 = time.time()
    ok = False
    while time.time() - t0 < TIMEOUT:
        try:
            body = pg.evaluate("document.body.innerText.slice(0,300)")
            if "发送验证码" not in body and (
                    "开始对话" in body or "新的对话" in body
                    or "deepseek" in body.lower().replace(" ", "")):
                # 二次确认：输入框存在
                if pg.locator("textarea").count() > 0:
                    ok = True
                    break
        except Exception:
            pass
        time.sleep(5)
    print("LOGIN-OK" if ok else "LOGIN-TIMEOUT")
    ctx.close()
