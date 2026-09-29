#!/usr/bin/env python3
"""nt_hf_rows_download — HF 数据集行采样 → 项目 datasets/hf_rows/ 目录.

走 datasets-server 行接口（与 `NtHfBridge::rows_endpoint` 同口径，
晶体摄入管线每次 100 条消费），只用标准库，零构建即可跑。

语义:
  - 默认取前 10 集（2026-09-24 trending 页头部）各前 N 行（默认 100）
  - 先试 config=default/split=train；失败则查 /splits 自动发现首个可用组合重试一次
  - 落点 `datasets/hf_rows/{owner}_{name}.rows{offset}-{offset+n}.json`（.tmp 原子写）
  - 已存在且为合法 rows JSON 则跳过；`--force` 强制重拉

用法（项目根执行，数据不出项目）:
  python3 scripts/ops/nt_hf_rows_download.py
  python3 scripts/ops/nt_hf_rows_download.py --rows 1000 --force
  python3 scripts/ops/nt_hf_rows_download.py --only wikimedia/wikipedia,nyu-mll/glue
  python3 scripts/ops/nt_hf_rows_download.py --ids venvoo/china-a-share-l2-level2-limit-order-book-tick-data,ikala/tmmluplus
  python3 scripts/ops/nt_hf_rows_download.py --selftest
"""
import argparse
import json
import os
import sys
import time
import urllib.request

ROWS_BASE = "https://datasets-server.huggingface.co/rows"
SPLITS_BASE = "https://datasets-server.huggingface.co/splits"
OUT_DIR = os.path.join("datasets", "hf_rows")
TIMEOUT = 60
UA = "NeoTrix/0.19 (nt_hf_rows_download.py)"

# trending 页头部 10 集（2026-09-24 快照，与 _index.json 头部一致）
FIRST10 = [
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
]


def api_get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
        return resp.status, resp.read()


def rows_url(ds_id, config, split, offset, length):
    return ("%s?dataset=%s&config=%s&split=%s&offset=%d&length=%d"
            % (ROWS_BASE, ds_id, config, split, offset, length))


def discover_splits(ds_id, limit=3):
    """查 /splits 取可用 (config, split) 列表，train 优先，最多 limit 个。"""
    try:
        status, body = api_get(SPLITS_BASE + "?dataset=" + ds_id)
        if status != 200:
            return []
        splits = json.loads(body).get("splits") or []
        trains = [(s.get("config", "default"), s.get("split", "train"))
                  for s in splits if s.get("split") == "train"]
        others = [(s.get("config", "default"), s.get("split", "train"))
                  for s in splits if s.get("split") != "train"]
        seen, out = set(), []
        for c in trains + others:
            if c not in seen:
                seen.add(c)
                out.append(c)
            if len(out) >= limit:
                break
        return out
    except Exception:  # noqa: BLE001 — 发现失败即回退，无备用则上SG8报失败
        return []


def valid_rows(path):
    try:
        with open(path, encoding="utf-8") as fh:
            d = json.load(fh)
        return isinstance(d, dict) and isinstance(d.get("rows"), list)
    except (OSError, ValueError):
        return False


def store_atomic(path, data):
    tmp = path + ".tmp"
    with open(tmp, "wb") as fh:
        fh.write(data)
    os.replace(tmp, path)


def fetch_one(ds_id, rows, offset, out_dir, force):
    dest = os.path.join(
        out_dir, "%s.rows%d-%d.json" % (ds_id.replace("/", "_"), offset, offset + rows))
    if not force and valid_rows(dest):
        return ("skip", dest)
    attempts = [("default", "train")]
    for cand in discover_splits(ds_id):
        if cand not in attempts:
            attempts.append(cand)
        if len(attempts) >= 3:
            break
    last_err = "unknown"
    for config, split in attempts:
        try:
            status, body = api_get(rows_url(ds_id, config, split, offset, rows))
            if status != 200:
                last_err = "HTTP %s (%s/%s)" % (status, config, split)
                continue
            d = json.loads(body)
            if not isinstance(d.get("rows"), list):
                last_err = "no rows field (%s/%s)" % (config, split)
                continue
            store_atomic(dest, body)
            return ("ok:%s/%s:%drows" % (config, split, len(d["rows"])), dest)
        except Exception as e:  # noqa: BLE001 — 逐集记录，不中断整批
            last_err = "%s (%s/%s)" % (str(e)[:100], config, split)
        time.sleep(1)
    return ("fail: " + last_err, dest)


def selftest():
    assert len(FIRST10) == 10 and len(set(FIRST10)) == 10
    u = rows_url("o/n", "default", "train", 0, 100)
    assert u.startswith(ROWS_BASE + "?dataset=o/n"), u
    print("selftest ok: 10 ids, rows url ok")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=100)
    ap.add_argument("--offset", type=int, default=0)
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--only", default="")
    ap.add_argument("--ids", default="")
    ap.add_argument("--output", default=OUT_DIR)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    ids = FIRST10
    if args.ids:
        ids = [i.strip() for i in args.ids.split(",") if i.strip()]
    elif args.only:
        want = set(args.only.split(","))
        ids = [i for i in FIRST10 if i in want]
    os.makedirs(args.output, exist_ok=True)
    results = []
    for ds_id in ids:
        how, dest = fetch_one(ds_id, args.rows, args.offset, args.output, args.force)
        print("%s %s" % (how.split(":")[0], ds_id))
        results.append((ds_id, how, dest))
        time.sleep(0.5)
    ok = sum(1 for _, h, _ in results if h.startswith("ok") or h == "skip")
    print("done: ok+skip=%d/%d" % (ok, len(results)))
    for ds_id, h, _ in results:
        if h.startswith("fail"):
            print("  FAIL %s: %s" % (ds_id, h))
    if any(h.startswith("fail") for _, h, _ in results):
        sys.exit(1)


if __name__ == "__main__":
    main()
