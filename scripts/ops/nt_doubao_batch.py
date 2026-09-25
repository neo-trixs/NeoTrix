#!/usr/bin/env python3
"""豆包视频总结批量采集（免登录 DOM 驱动，单会话复用）。"""
import json
import os
import sys
import time
from playwright.sync_api import sync_playwright

BASE = "datasets/douyin_woniu"
OUT = "%s/doubao_summaries.jsonl" % BASE
vs = json.load(open("%s/videos.json" % BASE, encoding="utf-8"))
done = set()
if os.path.exists(OUT):
    for ln in open(OUT, encoding="utf-8"):
        if ln.strip():
            done.add(json.loads(ln).get("id"))
todo = [v for v in vs if v["id"] not in done]
print("todo=", len(todo), flush=True)
n = 0
with sync_playwright() as p:
    b = p.chromium.launch(channel="chrome", headless=True)
    pg = b.new_page(
        user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                   "AppleWebKit/537.36 (KHTML, like Gecko) "
                   "Chrome/126.0.0.0 Safari/537.36")
    pg.goto("https://www.doubao.com/chat/", wait_until="domcontentloaded",
            timeout=60000)
    pg.wait_for_timeout(5000)
    with open(OUT, "a", encoding="utf-8") as fh:
        for v in todo:
            try:
                box = pg.locator('textarea, [contenteditable="true"]').first
                box.fill("请总结这个视频的内容：%s" % v["url"])
                pg.wait_for_timeout(800)
                pg.keyboard.press("Enter")
                pg.wait_for_timeout(24000)
                t = pg.evaluate("document.body.innerText")
                i = t.find("该视频")
                summary = t[i:i + 1200] if i >= 0 else ""
                if "登录" in t[-200:] and not summary:
                    print("LOGIN-WALL-HIT", v["id"], flush=True)
                    break
                fh.write(json.dumps({"id": v["id"], "title": v.get("title", ""),
                                     "summary": summary}, ensure_ascii=False) + "\n")
                fh.flush()
                n += 1
                print("ok %d/%d %s len=%d" % (n, len(todo), v["id"], len(summary)),
                      flush=True)
                pg.wait_for_timeout(2000)
            except Exception as e:
                print("FAIL", v["id"], str(e)[:80], flush=True)
    print("done new=", n)
    b.close()
