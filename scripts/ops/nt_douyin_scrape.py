#!/usr/bin/env python3
"""登录态抓取用户视频列表（复用扫码 profile，headless）。"""
import json
import sys
from playwright.sync_api import sync_playwright

UID = sys.argv[1] if len(sys.argv) > 1 else "80025557905"
PROFILE = "datasets/douyin_%s/profile" % UID
OUT = "datasets/douyin_%s/videos.json" % UID

with sync_playwright() as p:
    ctx = p.chromium.launch_persistent_context(
        PROFILE, channel="chrome", headless=True,
        args=["--disable-blink-features=AutomationControlled"])
    pg = ctx.pages[0] if ctx.pages else ctx.new_page()
    pg.goto("https://www.douyin.com/user/%s" % UID,
            wait_until="domcontentloaded", timeout=60000)
    pg.wait_for_timeout(6000)
    print("title=", pg.title())
    info = pg.evaluate("""() => ({
        nick: (document.querySelector('[data-e2e=user-info] h1, h1 span span')||{}).innerText || '',
        body: document.body.innerText.slice(0, 400)
    })""")
    print("nick=", info["nick"])
    print(info["body"][:400])
    # 滚动加载全部
    last = ""
    for _ in range(60):
        pg.mouse.wheel(0, 4000)
        pg.wait_for_timeout(1200)
        cur = pg.evaluate("document.body.innerHTML.length")
        if cur == last:
            break
        last = cur
    links = pg.eval_on_selector_all(
        'a[href*="/video/"]',
        "els => els.map(e => ({url: e.href, title: e.getAttribute('title') || e.innerText || ''}))")
    uniq = {}
    for l in links:
        vid = l["url"].split("/video/")[1].split("?")[0].split("/")[0]
        if vid.isdigit():
            uniq[vid] = l["title"].strip()[:80]
    print("videos-found=", len(uniq))
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump([{"id": v, "title": t,
                    "url": "https://www.douyin.com/video/%s" % v}
                   for v, t in uniq.items()], fh, ensure_ascii=False, indent=1)
    print("saved:", OUT)
    ctx.close()
