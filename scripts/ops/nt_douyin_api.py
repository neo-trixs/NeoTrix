#!/usr/bin/env python3
"""登录态打开视频页，拦截官方 API，拿作者 sec_uid 与视频直链。"""
import json
import sys
from playwright.sync_api import sync_playwright

URL = sys.argv[1]
OUT = sys.argv[2]
PROFILE = "datasets/douyin_80025557905/profile"
captured = {}


def on_resp(resp):
    try:
        u = resp.url
        if "aweme/v1/web/aweme/detail" in u or "aweme/v1/web/aweme/post" in u:
            captured.setdefault("detail" if "detail" in u else "post", []).append(resp.json())
    except Exception:
        pass


with sync_playwright() as p:
    ctx = p.chromium.launch_persistent_context(
        PROFILE, channel="chrome", headless=True,
        args=["--disable-blink-features=AutomationControlled"])
    pg = ctx.pages[0] if ctx.pages else ctx.new_page()
    pg.on("response", on_resp)
    pg.goto(URL, wait_until="domcontentloaded", timeout=60000)
    pg.wait_for_timeout(9000)
    print("title=", pg.title()[:60])
    print("captured=", {k: len(v) for k, v in captured.items()})
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump(captured, fh, ensure_ascii=False)
    print("saved:", OUT)
    ctx.close()
