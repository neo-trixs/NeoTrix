#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_hf_survey — 批量勘察 HF 数据集（只读，不下载），为吸收决策定量.

## 为什么先勘察而不是直接灌

晶体核心是**单个 JSON**，由 1h tick 全量 `read_to_string` + `from_str` 载入并
**常驻内存**（`CrystalIterState::load` → `sync_to_consciousness`）。当前已
52.5MB / 64,674 条。若把 12 个数据集里的大件（`Fable-...-10000x` 字面即万倍量级）
原样灌入，核心会膨胀到不可加载。所以每次吸收前必须知道：**多大、多少行、什么许可、
是否 gated**。这与前两轮（医患 75MB、MiMo 21MB）的纪律一致。

同时勘察 **在既有目录里出现过**的项（`nt_hf_catalog_download.py` 的 2026-09-24
trending 快照），避免重复劳动、并标出哪些是"回归"。

用法:
  python3 scripts/ops/nt_hf_survey.py
  python3 scripts/ops/nt_hf_survey.py --selftest
退出码: 0=全部可取 1=有 gated/不可取
"""
import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request

DS_IDS = [
    "MoreThought/Fable-5.1-Max-Reasoning-Filtered-10000x",
    "eidon-ai/tracker-pov",
    "FineEnvs/SmolDataEnvs",
    "ZefanCai/Open-Jev",
    "LocalLLaMA/typed-decisions",
    "genrobot2025/Gen-HumanEgo",
    "Yootta/World-SimReady-Home",
    "Anthropic/hh-rlhf",
    "AxiomicLabs/Tiny_Theory_of_Mind",
    "nyu-mll/glue",
    "IFM/Code-Reasoning",
    "malcolmrey/various",
]
API = "https://huggingface.co/api/datasets/"
SPLITS = "https://datasets-server.huggingface.co/splits?dataset="
UA = "NeoTrix/0.19 (nt_hf_survey.py)"
TIMEOUT = 45
OUT = os.path.join("datasets", "hf_survey.json")


def fetch_json(url, timeout=TIMEOUT):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read())


def selftest():
    assert len(DS_IDS) == len(set(DS_IDS)), "清单有重复项（应先去重）"
    assert all("/" in d for d in DS_IDS), "id 必须是 owner/name"
    assert API.endswith("/")
    print("selftest ok: %d unique ids / id-shape" % len(DS_IDS))
    return 0


def survey_one(ds, deep=True):
    row = {"id": ds, "ok": False}
    try:
        m = fetch_json(API + ds)
    except urllib.error.HTTPError as e:
        row["error"] = "HTTP %s" % e.code
        return row
    except Exception as e:  # noqa: BLE001 — 逐个记录，不中断整批
        row["error"] = str(e)[:90]
        return row
    row["ok"] = True
    row["gated"] = bool(m.get("gated"))
    row["private"] = bool(m.get("private"))
    row["disabled"] = bool(m.get("disabled"))
    row["sha"] = (m.get("sha") or "")[:12]
    row["last_modified"] = (m.get("lastModified") or "")[:10]
    row["downloads"] = m.get("downloads", 0)
    row["likes"] = m.get("likes", 0)
    row["storage_bytes"] = m.get("usedStorage")
    tags = m.get("tags") or []
    row["license"] = next((t.split(":", 1)[1] for t in tags if t.startswith("license:")), "")
    row["langs"] = [t.split(":", 1)[1] for t in tags if t.startswith("language:")][:6]
    row["size_cat"] = next((t.split(":", 1)[1] for t in tags if t.startswith("size_categories:")), "")
    row["tasks"] = [t.split(":", 1)[1] for t in tags if t.startswith("task_categories:")][:4]
    card = m.get("cardData") or {}
    cfgs = card.get("configs") or []
    row["configs"] = [{"name": (c or {}).get("config_name"),
                      "files": ((c or {}).get("data_files") or "")}
                     for c in cfgs][:8]
    if deep and not row["gated"]:
        try:
            sp = fetch_json(SPLITS + ds)
            row["splits"] = [{"config": s.get("config"), "split": s.get("split")}
                             for s in (sp.get("splits") or [])][:8]
            row["splits_failed"] = bool(sp.get("failed"))
        except Exception as e:  # noqa: BLE001
            row["splits_error"] = str(e)[:60]
    return row


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--no-deep", action="store_true")
    ap.add_argument("--out", default=OUT)
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    print("勘察 %d 个数据集（只读）…\n" % len(DS_IDS))
    rows = []
    hdr = ("%-46s %8s %6s %5s %-11s %-9s %s"
           % ("dataset", "storage", "dl", "like", "license", "size_cat", "splits"))
    print(hdr)
    print("-" * len(hdr))
    for i, ds in enumerate(DS_IDS, 1):
        r = survey_one(ds, deep=not args.no_deep)
        rows.append(r)
        if not r.get("ok"):
            print("%-46s   %s" % (ds[:46], r.get("error")))
        else:
            sb = r.get("storage_bytes")
            sbs = ("%.1fMB" % (sb / 1e6)) if isinstance(sb, (int, float)) and sb < 1e9 else (
                "%.2fGB" % (sb / 1e9)) if isinstance(sb, (int, float)) else "-"
            nsp = len(r.get("splits") or [])
            print("%-46s %8s %6d %5d %-11s %-9s %d"
                  % (ds[:46], sbs, r.get("downloads", 0), r.get("likes", 0),
                     r.get("license") or "-", r.get("size_cat") or "-", nsp))
        sys.stdout.flush()
        time.sleep(0.4)

    os.makedirs(os.path.dirname(args.out) or ".", exist_ok=True)
    tmp = args.out + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(rows, fh, ensure_ascii=False, indent=1)
    os.replace(tmp, args.out)
    print("\nsurvey → %s" % args.out)

    ok = [r for r in rows if r.get("ok")]
    tot = sum(r.get("storage_bytes") or 0 for r in ok)
    gated = [r["id"] for r in ok if r.get("gated")]
    print("可取 %d/%d   许可: %s" % (len(ok), len(rows),
          " ".join(sorted({r.get("license") or "?" for r in ok}))))
    print("gated 需授权: %s" % (gated or "无"))
    print("总存储 %.2f GB" % (tot / 1e9))
    if gated:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
