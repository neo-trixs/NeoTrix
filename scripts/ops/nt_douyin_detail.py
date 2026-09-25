#!/usr/bin/env python3
"""逐视频页拦截 aweme/detail API，存 JSON（play直链+描述+作者）。"""
import json
import os
import sys
from playwright.sync_api import sync_playwright

BASE = "datasets/douyin_woniu"
PROFILE = "datasets/douyin_80025557905/profile"
os.makedirs("%s/detail" % BASE, exist_ok=True)
vs = json.load(open("%s/videos.json" % BASE, encoding="utf-8"))
todo = [v for v in vs
        if not os.path.exists("%s/detail/%s.json" % (BASE, v["id"]))]
print("todo=", len(todo))
got = {}


def on_resp(resp):
    try:
        if "aweme/v1/web/aweme/detail" in resp.url:
            got["last"] = resp.json()
    except Exception:
        pass


with sync_playwright() as p:
    ctx = p.chromium.launch_persistent_context(
        PROFILE, channel="chrome", headless=True,
        args=["--disable-blink-features=AutomationControlled"])
    pg = ctx.pages[0] if ctx.pages else ctx.new_page()
    pg.on("response", on_resp)
    ok = 0
    for v in todo:
        got.clear()
        try:
            pg.goto(v["url"], wait_until="domcontentloaded", timeout=45000)
            pg.wait_for_timeout(4000)
            if "last" in got:
                with open("%s/detail/%s.json" % (BASE, v["id"]), "w",
                          encoding="utf-8") as fh:
                    json.dump(got["last"], fh, ensure_ascii=False)
                ok += 1
            else:
                print("NOAPI", v["id"])
        except Exception as e:
            print("FAIL", v["id"], str(e)[:60])
    print("saved-detail=", ok)
    ctx.close()
