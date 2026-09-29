#!/usr/bin/env python3
"""nt_hf_rows_to_crawl — 行采样 → crawl_queue 行，一步进晶体管线.

映射口径对齐 `NtHfBridge::{rows_to_memories, to_crawl_queue_lines}`，
另加两处行级扩展（Rust 桥后续同改，构建开锁后）：
  - prompt 键扩展 sentence/conversation/text/content/body（wikipedia/glue/saas 直收）
  - 纯数值行兜底为 k=v 摘要行（arxiv manifest / tracker 特征行不丢数）

产出:
  - 项目存档 `datasets/hf_rows/crawl_queue.jsonl`（全量 700 行级，可审计）
  - 追加到晶体活队列 `~/.neotrix/crawl_queue.jsonl`（先备份，已存在 url 去重；
    活队列是晶体运行时嘴（与 crystal_state.json 同家），项目数据仍全在 datasets/）
  - 消费：`IngestionEngine::ingest_crawl_queue` 每轮 100 条 → crystal remember

用法:
  python3 scripts/ops/nt_hf_rows_to_crawl.py
  python3 scripts/ops/nt_hf_rows_to_crawl.py --no-push   # 只写项目存档，不碰活队列
  python3 scripts/ops/nt_hf_rows_to_crawl.py --selftest
"""
import argparse
import glob
import json
import os
import sys

ROWS_DIR = os.path.join("datasets", "hf_rows")
ARCHIVE = os.path.join(ROWS_DIR, "crawl_queue.jsonl")
LIVE = os.path.expanduser("~/.neotrix/crawl_queue.jsonl")
DOMAIN = "hf-rows"

PROMPT_KEYS = ["instruction", "question", "problem", "input", "prompt", "query",
               "messages", "conversations",
               "sentence", "conversation", "text", "content", "body"]
COMPLETION_KEYS = ["output", "answer", "response", "completion", "label", "target"]
THINK_KEYS = ["reasoning", "thought", "thinking", "chain_of_thought", "rationale"]


def first_str(row, keys):
    for k in keys:
        v = row.get(k)
        if isinstance(v, str) and v.strip():
            return v
        if isinstance(v, list) and v and all(isinstance(m, dict) for m in v):
            parts = []
            for m in v:
                role = m.get("role", "")
                text = m.get("content", "")
                if isinstance(text, str) and text.strip():
                    parts.append("[%s] %s" % (role, text))
            if parts:
                return "\n".join(parts)
    return ""


def summary_fallback(row):
    bits = []
    for k, v in row.items():
        if isinstance(v, (str, int, float, bool)) and v != "" and v is not None:
            bits.append("%s=%s" % (k, str(v)[:80]))
        elif isinstance(v, dict) and isinstance(v.get("src"), str):
            bits.append("%s=%s" % (k, v["src"][:160]))
        if len(bits) >= 8:
            break
    return "；".join(bits)


def row_to_line(ds_id, idx, row):
    prompt = first_str(row, PROMPT_KEYS)
    completion = first_str(row, COMPLETION_KEYS)
    thinking = first_str(row, THINK_KEYS)
    if not prompt and not completion:
        prompt = summary_fallback(row)
    if not prompt and not completion:
        return None
    title = (prompt[:120] if len(prompt) > 120 else prompt)
    if completion:
        content = "问：%s\n答：%s" % (prompt, completion)
    else:
        content = prompt
    if thinking:
        content += "\n思考：%s" % thinking[:500]
    return json.dumps({
        "url": "hf-dataset://%s#row%d" % (ds_id, idx),
        "title": title,
        "content": content,
        "domain": DOMAIN,
    }, ensure_ascii=False)


def convert(rows_dir=ROWS_DIR):
    lines, per_ds = [], {}
    for f in sorted(glob.glob(os.path.join(rows_dir, "*.rows*.json"))):
        ds_id = os.path.basename(f).split(".rows")[0].replace("_", "/", 1)
        try:
            with open(f, encoding="utf-8") as fh:
                d = json.load(fh)
        except (OSError, ValueError):
            per_ds[ds_id] = "bad file"
            continue
        kept = 0
        for i, item in enumerate(d.get("rows", [])):
            row = item.get("row", {}) if isinstance(item, dict) else {}
            line = row_to_line(ds_id, i, row) if isinstance(row, dict) else None
            if line:
                lines.append(line)
                kept += 1
        per_ds[ds_id] = "%d kept" % kept
    return lines, per_ds


def push_live(lines):
    existing = set()
    if os.path.exists(LIVE):
        with open(LIVE, encoding="utf-8") as fh:
            for ln in fh:
                try:
                    u = json.loads(ln).get("url", "")
                except ValueError:
                    continue
                if u.startswith("hf-dataset://"):
                    existing.add(u)
    new = [ln for ln in lines if json.loads(ln)["url"] not in existing]
    if not new:
        return 0, len(existing), False
    backed = False
    if os.path.exists(LIVE):
        bak = LIVE + ".bak.hfrows"
        with open(LIVE, "rb") as src, open(bak, "wb") as dst:
            dst.write(src.read())
        backed = True
    with open(LIVE, "a", encoding="utf-8") as fh:
        for ln in new:
            fh.write(ln + "\n")
    return len(new), len(existing), backed


def selftest():
    m = {"messages": [{"role": "user", "content": "hi"},
                      {"role": "assistant", "content": "hello"}]}
    ln = row_to_line("o/n", 0, m)
    assert ln and "[user] hi" in ln and "hf-dataset://o/n#row0" in ln, ln
    g = {"sentence": "It is cold.", "label": "0", "idx": 1}
    ln = row_to_line("o/g", 3, g)
    assert "It is cold." in ln and "答：0" in ln, ln
    w = {"id": "1", "title": "T", "text": "Body here", "url": "u"}
    assert "Body here" in row_to_line("o/w", 0, w)
    n = {"kind": "f", "size": 12, "sha256": "ab"}
    assert "kind=f" in row_to_line("o/a", 0, n)
    assert row_to_line("o/e", 0, {}) is None
    print("selftest ok: messages/glue/wiki/numeric/empty")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-push", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    lines, per_ds = convert()
    with open(ARCHIVE, "w", encoding="utf-8") as fh:
        for ln in lines:
            fh.write(ln + "\n")
    for ds, st in per_ds.items():
        print("%s: %s" % (ds, st))
    print("archive: %s (%d lines)" % (ARCHIVE, len(lines)))
    if args.no_push:
        return
    added, dup, backed = push_live(lines)
    print("live queue: %s added=%d dup_skipped=%d backup=%s"
          % (LIVE, added, dup, backed))
    if not lines:
        sys.exit(1)


if __name__ == "__main__":
    main()
