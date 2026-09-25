#!/usr/bin/env python3
"""抓取抖音用户主页视频列表（playwright + 系统 Chrome headless）。"""
import json
import sys
from playwright.sync_api import sync_playwright

UID = sys.argv[1] if len(sys.argv) > 1 else "80025557905"
OUT = sys.argv[2] if len(sys.argv) > 2 else "datasets/douyin_%s/videos.json" % UID

with sync_playwright() as p:
    b = p.chromium.launch(channel="chrome", headless=True,
                          args=["--disable-blink-features=AutomationControlled"])
    pg = b.new_page(user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    "Chrome/126.0.0.0 Safari/537.36")
    pg.goto("https://www.douyin.com/user/%s" % UID, wait_until="domcontentloaded",
            timeout=60000)
    pg.wait_for_timeout(8000)
    pg.keyboard.press("Escape")
    pg.wait_for_timeout(1000)
    # 关登录弹窗（右上 ×）
    for sel in ["div[role='dialog'] svg", "svg path[d*='M7 7']",
                "div:has-text('登录后免费畅享高清视频') svg"]:
        try:
            pg.click(sel, timeout=2500)
            print("closed modal via", sel)
            break
        except Exception as e:
            print("close try failed:", sel)
    pg.wait_for_timeout(3000)
    pg.screenshot(path="datasets/douyin_%s/home.png" % UID)
    # 关登录弹窗（若有）
    for sel in ["div[class*='login'] svg", "i[class*='close']", ".dy-account-close"]:
        try:
            pg.click(sel, timeout=2000)
            break
        except Exception:
            pass
    # 滚动加载
    seen, last = set(), ""
    for _ in range(40):
        pg.mouse.wheel(0, 3000)
        pg.wait_for_timeout(1500)
        html_len = pg.evaluate("document.body.innerHTML.length")
        if html_len == last:
            break
        last = html_len
    links = pg.eval_on_selector_all(
        'a[href*="/video/"]',
        "els => els.map(e => ({url: e.href, title: e.getAttribute('title') || e.innerText || ''}))")
    uniq = {}
    for l in links:
        vid = l["url"].split("/video/")[1].split("?")[0].split("/")[0]
        if vid.isdigit():
            uniq[vid] = l["title"].strip()[:80]
    print("videos-found=", len(uniq))
    print("page-title=", pg.title())
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump([{"id": v, "title": t,
                    "url": "https://www.douyin.com/video/%s" % v}
                   for v, t in uniq.items()], fh, ensure_ascii=False, indent=1)
    print("saved:", OUT)
    b.close()
