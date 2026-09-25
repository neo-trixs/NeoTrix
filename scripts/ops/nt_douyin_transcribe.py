#!/usr/bin/env python3
"""faster-whisper 批量转写 media/manifest.jsonl → transcripts.jsonl（断点续跑）。"""
import json
import os
import sys

BASE = "datasets/douyin_woniu"
MANIFEST = "%s/media/manifest.jsonl" % BASE
OUT = "%s/transcripts.jsonl" % BASE
MODEL = sys.argv[1] if len(sys.argv) > 1 else "base"

from faster_whisper import WhisperModel
model = WhisperModel(MODEL, device="cpu", compute_type="int8")
print("model loaded:", MODEL, flush=True)

done = set()
if os.path.exists(OUT):
    for ln in open(OUT, encoding="utf-8"):
        if ln.strip():
            done.add(json.loads(ln).get("id"))
print("already done:", len(done), flush=True)

items = [json.loads(l) for l in open(MANIFEST, encoding="utf-8") if l.strip()]
todo = [it for it in items if it["id"] not in done]
print("todo=", len(todo), flush=True)
n = 0
with open(OUT, "a", encoding="utf-8") as fh:
    for it in todo:
        segs, info = model.transcribe(it["mp4"], language="zh", beam_size=5)
        text = "".join(s.text for s in segs).strip()
        fh.write(json.dumps({"id": it["id"], "title": it.get("title", ""),
                             "desc": it.get("desc", ""),
                             "nickname": it.get("nickname", ""),
                             "duration": info.duration, "text": text},
                            ensure_ascii=False) + "\n")
        fh.flush()
        n += 1
        print("ok %d/%d %s chars=%d" % (n, len(todo), it["id"], len(text)), flush=True)
print("done new=", n)
