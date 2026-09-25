#!/usr/bin/env python3
"""登录态作者主页：滚动收全部视频 ID + 逐页存 HTML（供 Rust 管道解析）。"""
import json
import sys
from playwright.sync_api import sync_playwright

SEC_UID = sys.argv[1]
BASE = "datasets/douyin_woniu"
PROFILE = "datasets/douyin_80025557905/profile"
posts = []


def on_resp(resp):
    try:
        if "aweme/v1/web/aweme/post" in resp.url:
            posts.append(resp.json())
    except Exception:
        pass


with sync_playwright() as p:
    ctx = p.chromium.launch_persistent_context(
        PROFILE, channel="chrome", headless=True,
        args=["--disable-blink-features=AutomationControlled"])
    pg = ctx.pages[0] if ctx.pages else ctx.new_page()
    pg.on("response", on_resp)
    pg.goto("https://www.douyin.com/user/%s" % SEC_UID,
            wait_until="domcontentloaded", timeout=60000)
    pg.wait_for_timeout(6000)
    print("title=", pg.title()[:50])
    last = ""
    for _ in range(80):
        pg.mouse.wheel(0, 4000)
        pg.wait_for_timeout(1200)
        cur = pg.evaluate("document.body.innerHTML.length")
        if cur == last:
            break
        last = cur
    # DOM 收集
    links = pg.eval_on_selector_all(
        'a[href*="/video/"]',
        "els => els.map(e => ({url: e.href, title: e.getAttribute('title') || ''}))")
    uniq = {}
    for l in links:
        vid = l["url"].split("/video/")[1].split("?")[0].split("/")[0]
        if vid.isdigit():
            uniq[vid] = l["title"].strip()[:80]
    # API 收集
    for pack in posts:
        for a in (pack.get("aweme_list") or []):
            vid = str(a.get("aweme_id", ""))
            if vid.isdigit() and vid not in uniq:
                uniq[vid] = (a.get("desc") or "")[:80]
    print("dom+api videos=", len(uniq), "api-packs=", len(posts))
    with open("%s/videos.json" % BASE, "w", encoding="utf-8") as fh:
        json.dump([{"id": v, "title": t,
                    "url": "https://www.douyin.com/video/%s" % v}
                   for v, t in uniq.items()], fh, ensure_ascii=False, indent=1)
    print("saved videos.json")
    ctx.close()
