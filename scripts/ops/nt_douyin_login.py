#!/usr/bin/env python3
"""扫码登录：可见浏览器 + 持久 profile，轮询登录态。"""
import sys
import time
from playwright.sync_api import sync_playwright

UID = sys.argv[1] if len(sys.argv) > 1 else "80025557905"
PROFILE = "datasets/douyin_%s/profile" % UID
TIMEOUT = int(sys.argv[2]) if len(sys.argv) > 2 else 300

with sync_playwright() as p:
    ctx = p.chromium.launch_persistent_context(
        PROFILE, channel="chrome", headless=False,
        args=["--disable-blink-features=AutomationControlled"],
        user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                   "AppleWebKit/537.36 (KHTML, like Gecko) "
                   "Chrome/126.0.0.0 Safari/537.36",
        viewport={"width": 1400, "height": 900})
    pg = ctx.pages[0] if ctx.pages else ctx.new_page()
    pg.goto("https://www.douyin.com/user/%s" % UID,
            wait_until="domcontentloaded", timeout=60000)
    print("请用抖音APP扫码登录（等待%d秒）…" % TIMEOUT)
    t0 = time.time()
    ok = False
    while time.time() - t0 < TIMEOUT:
        try:
            cookies = {c["name"] for c in ctx.cookies()}
            body = pg.evaluate("document.body.innerText.slice(0,300)")
            if "sessionid" in cookies and "登录后免费" not in body:
                ok = True
                break
        except Exception:
            pass
        time.sleep(5)
    print("LOGIN-OK" if ok else "LOGIN-TIMEOUT")
    try:
        pg.screenshot(path="datasets/douyin_%s/login.png" % UID)
    except Exception:
        pass
    ctx.close()
