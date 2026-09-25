#!/usr/bin/env python3
"""DeepSeek 网页自调用批量蒸馏：DOM 发送 + SSE 响应直读（不爬 DOM）。
用法：python3 scripts/ops/nt_deepseek_batch.py
输入：datasets/douyin_woniu/transcripts.jsonl
输出：datasets/douyin_woniu/deepseek_distill.jsonl
"""
import json
import os
import sys
import time
from playwright.sync_api import sync_playwright

BASE = "datasets/douyin_woniu"
PROFILE = "datasets/deepseek/profile"
OUT = "%s/deepseek_distill.jsonl" % BASE
CHUNK = 4


def parse_sse(text):
    base, app = [], []
    for ln in text.splitlines():
        ln = ln.strip()
        if not ln.startswith("data:"):
            continue
        try:
            d = json.loads(ln[5:])
        except Exception:
            continue
        if d.get("o") == "APPEND" and isinstance(d.get("v"), str):
            app.append(d["v"])
            continue
        v = d.get("v", {})
        if not isinstance(v, dict):
            continue
        resp = v.get("response", {})
        if not isinstance(resp, dict):
            continue
        for f in resp.get("fragments", []) or []:
            if isinstance(f, dict) and f.get("content"):
                base.append(f["content"])
    return "".join(base) + "".join(app)


def main():
    vs = [json.loads(l) for l in
          open("%s/transcripts.jsonl" % BASE, encoding="utf-8") if l.strip()]
    done_ids = set()
    if os.path.exists(OUT):
        for ln in open(OUT, encoding="utf-8"):
            if ln.strip():
                done_ids.add(json.loads(ln).get("chunk"))
    chunks = [vs[i:i + CHUNK] for i in range(0, len(vs), CHUNK)]
    want = sys.argv[1:] and [int(a) for a in sys.argv[1:]] or None
    todo = [(i, c) for i, c in enumerate(chunks)
            if i not in done_ids and (want is None or i in want)]
    print("todo-chunks=", len(todo), flush=True)
    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            PROFILE, channel="chrome", headless=True)
        pg = ctx.pages[0] if ctx.pages else ctx.new_page()
        pg.goto("https://chat.deepseek.com/",
                wait_until="domcontentloaded", timeout=60000)
        pg.wait_for_timeout(5000)
        with open(OUT, "a", encoding="utf-8") as fh:
            for ci, chunk in todo:
                texts = "\n\n".join(
                    "[%s] %s" % (x["id"], x["text"][:800]) for x in chunk)
                prompt = ("以下是%d段短视频语音转写文本（每段约300字）（有同音字错误）。"
                          "请做三件事：1）列出每段ID的一句话核心观点；"
                          "2）把全部段落按主题聚成3-6类并命名；"
                          "3）指出转写中明显的同音字错误并纠正（如唯独/维度）。"
                          "直接给结论，不要寒暄。\n\n%s" % (len(chunk), texts))
                box = {"texts": [], "done": False}

                def on_rsp(r):
                    if "chat/completion" in r.url:
                        try:
                            box["texts"].append(r.text())
                        except Exception:
                            pass

                def on_fin(req):
                    if "chat/completion" in req.url:
                        box["done"] = True

                pg.on("response", on_rsp)
                pg.on("requestfinished", on_fin)
                pg.locator("textarea").first.click(force=True)
                pg.wait_for_timeout(500)
                # 分段粘贴防超长单次输入
                for k in range(0, len(prompt), 3000):
                    pg.keyboard.type(prompt[k:k + 3000])
                    pg.wait_for_timeout(300)
                pg.wait_for_timeout(500)
                pg.keyboard.press("Enter")
                # 等待：答案长度>200 且连续3次(20s间隔)零增长；超时10分钟
                answer, t0, stable = "", time.time(), 0
                while time.time() - t0 < 590:
                    pg.wait_for_timeout(20000)
                    cur = ""
                    if box["texts"]:
                        try:
                            cur = parse_sse(box["texts"][-1])
                        except Exception:
                            pass
                    if len(cur) > 200 and len(cur) == len(answer):
                        stable += 1
                        if stable >= 3:
                            answer = cur
                            break
                    else:
                        stable = 0
                    answer = cur
                fh.write(json.dumps(
                    {"chunk": ci, "ids": [x["id"] for x in chunk],
                     "answer": answer}, ensure_ascii=False) + "\n")
                fh.write(json.dumps(
                    {"chunk": ci, "ids": [x["id"] for x in chunk],
                     "answer": answer}, ensure_ascii=False) + "\n")
                fh.flush()
                print("chunk %d done len=%d" % (ci, len(answer)), flush=True)
                pg.wait_for_timeout(3000)
        ctx.close()


if __name__ == "__main__":
    main()
