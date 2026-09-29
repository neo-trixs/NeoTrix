#!/usr/bin/env python3
"""nt_hf_catalog_download — HF datasets 广场目录元数据 → 项目 datasets/ 目录.

只用标准库（urllib），零构建即可跑；Rust 侧生产路径见
`neotrix-core/src/bin/nt_hf_catalog_download.rs`（走 nt_http，同语义：
原子落盘 + 幂等跳过）。本脚本与 Rust bin 共用同一清单与落点，
构建锁被占时先用本脚本下载，后补构建验证。

语义:
  - 每个数据集拉 `GET /api/datasets/{owner}/{name}` 元数据 JSON
  - 落点 `datasets/hf_catalog/{owner}_{name}.json`（.tmp + os.replace 原子写）
  - 已存在且为合法 JSON（含 id）则跳过；`--force` 强制重拉
  - 结束重建 `_index.json` 紧凑索引（id/likes/downloads/tasks/模态/格式/许可/语言/摘要）

用法（项目根执行，数据不出项目）:
  python3 scripts/ops/nt_hf_catalog_download.py
  python3 scripts/ops/nt_hf_catalog_download.py --force
  python3 scripts/ops/nt_hf_catalog_download.py --selftest
"""
import argparse
import json
import os
import sys
import urllib.request

API_BASE = "https://huggingface.co/api/datasets/"
OUT_DIR = os.path.join("datasets", "hf_catalog")
INDEX_NAME = "_index.json"
TIMEOUT = 30
UA = "NeoTrix/0.19 (nt_hf_catalog_download.py)"

# 2026-09-24 https://huggingface.co/datasets 首屏 trending 30 集快照
DATASET_IDS = [
    "secemp9/arxiv-complete",
    "MoreThought/Fable-5.1-Max-Reasoning-Filtered-5000x",
    "wikimedia/wikipedia",
    "openbmb/UltraData-SFT-Agent-2609",
    "Yootta/World-SimReady-Home",
    "zgcagi/ZGCM-1-Data",
    "markov-ai/cad-1000-hours",
    "DeepMostInnovations/saas-sales-conversations",
    "eidon-ai/tracker-pov",
    "nyu-mll/glue",
    "venvoo/china-a-share-l2-level2-limit-order-book-tick-data",
    "malcolmrey/various",
    "ikala/tmmluplus",
    "OpenDataArena/Spark-234K",
    "FlyRank/internship-warehouse",
    "openbmb/UltraData-Code",
    "Harland/OmniVChat",
    "ZefanCai/Open-Jev",
    "Anthropic/hh-rlhf",
    "nvidia/PhysicalAI-Autonomous-Vehicles",
    "nvidia/OpenH-RF",
    "IFM/Code-Reasoning",
    "echel0nn1881/kimi-cyber-reasoning",
    "LocalLLaMA/typed-decisions",
    "ILSVRC/imagenet-1k",
    "HuggingFaceFW/fineweb",
    "IFM/TxT360-v2",
    "openai/gsm8k",
    "saidutta69/fable-5-premium",
    "openbmb/UltraData-RL-2609",
]


def dest_of(out_dir, ds_id):
    return os.path.join(out_dir, ds_id.replace("/", "_") + ".json")


def valid_catalog(path):
    try:
        with open(path, encoding="utf-8") as fh:
            d = json.load(fh)
        return isinstance(d, dict) and isinstance(d.get("id"), str) and "/" in d["id"]
    except (OSError, ValueError):
        return False


def fetch(ds_id):
    req = urllib.request.Request(
        API_BASE + ds_id, headers={"Accept": "application/json", "User-Agent": UA}
    )
    with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
        if resp.status != 200:
            raise RuntimeError("HTTP %s" % resp.status)
        return resp.read()


def store_atomic(path, data):
    tmp = path + ".tmp"
    with open(tmp, "wb") as fh:
        fh.write(data)
    os.replace(tmp, path)


def pick(tags, prefix, limit=0):
    out = [t.split(":", 1)[1] for t in tags if t.startswith(prefix + ":")]
    return out[:limit] if limit else out


def rebuild_index(out_dir):
    items = []
    for fn in sorted(os.listdir(out_dir)):
        if not fn.endswith(".json") or fn.startswith("_"):
            continue
        try:
            with open(os.path.join(out_dir, fn), encoding="utf-8") as fh:
                d = json.load(fh)
            if not isinstance(d, dict) or "id" not in d:
                continue
            tags = d.get("tags") or []
            lic = pick(tags, "license")
            items.append({
                "id": d["id"],
                "likes": d.get("likes", 0),
                "downloads": d.get("downloads", 0),
                "tasks": pick(tags, "task_categories"),
                "modality": pick(tags, "modality"),
                "formats": pick(tags, "format"),
                "licenses": lic,
                "langs": pick(tags, "language", 8),
                "desc": (d.get("description") or "")[:300],
            })
        except (OSError, ValueError):
            continue
    index = {
        "snapshot": "2026-09-24",
        "source": "https://huggingface.co/datasets trending p0",
        "count": len(items),
        "items": items,
    }
    with open(os.path.join(out_dir, INDEX_NAME), "w", encoding="utf-8") as fh:
        json.dump(index, fh, ensure_ascii=False, indent=1)
    return len(items)


def selftest():
    fixture = ('{"id":"o/n","likes":12,"downloads":34,"tags":'
               '["task_categories:text-generation","license:mit"],'
               '"description":"  hello  "}')
    d = json.loads(fixture)
    assert d["id"] == "o/n"
    assert pick(d["tags"], "task_categories") == ["text-generation"]
    assert pick(d["tags"], "license") == ["mit"]
    assert len(DATASET_IDS) == 30, len(DATASET_IDS)
    assert len(set(DATASET_IDS)) == 30
    print("selftest ok: 30 ids, tag pick ok")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--output", default=OUT_DIR)
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    os.makedirs(args.output, exist_ok=True)
    ok, skipped, failed = 0, 0, []
    for ds_id in DATASET_IDS:
        dest = dest_of(args.output, ds_id)
        if not args.force and valid_catalog(dest):
            skipped += 1
            continue
        try:
            store_atomic(dest, fetch(ds_id))
            if valid_catalog(dest):
                ok += 1
                print("ok %s" % ds_id)
            else:
                failed.append(ds_id + " (bad json)")
                print("FAIL %s (bad json)" % ds_id)
        except Exception as e:  # noqa: BLE001 — 逐个记录，不中断整批
            failed.append("%s (%s)" % (ds_id, str(e)[:100]))
            print("FAIL %s: %s" % (ds_id, str(e)[:120]))
    n = rebuild_index(args.output)
    print("done: downloaded=%d skipped=%d fail=%d index=%d" % (ok, skipped, len(failed), n))
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
