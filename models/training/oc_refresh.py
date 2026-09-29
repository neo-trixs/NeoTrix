#!/usr/bin/env python3
"""OpenChamber 每日刷新（实时信息能力）：cards.json 是日更静态文件，
重下 → 按 tweet id 差分 → 新仓并入熔炼索引 + 新卡 append JEV 记忆。
用法：python3 models/training/oc_refresh.py [--top N]
cron: 0 9 * * * cd /path/to/neotrix && python3 models/training/oc_refresh.py
"""
import argparse
import json
import os
import re
import subprocess
import sys

OC = "https://jev.openchamber.dev"
_SMELT = os.path.join(os.path.dirname(__file__), "smelt")
SEEN = os.path.join(_SMELT, "oc_seen_ids.json")
INDEX = os.path.join(_SMELT, "repo_index.json")


def fetch(path, out):
    r = subprocess.run(
        ["curl", "-sL", "--max-time", "60", f"{OC}/{path}", "-o", out],
        capture_output=True,
    )
    return r.returncode == 0 and os.path.exists(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--top", type=int, default=1500)
    ap.add_argument("--repo", dest="repo", default=os.getcwd())
    args = ap.parse_args()

    tmp = os.path.join(_SMELT, "oc_cards_latest.json")
    if not fetch("data/cards.json", tmp):
        print("[oc] download failed", flush=True)
        sys.exit(1)
    d = json.load(open(tmp, encoding="utf-8"))
    print(f"[oc] meta: {json.dumps(d.get('meta', {}))[:200]}", flush=True)
    cs = d.get("cards", [])
    seen = set(json.load(open(SEEN)) if os.path.exists(SEEN) else [])
    fresh = [c for c in cs if str(c.get("id")) not in seen]
    print(f"[oc] cards={len(cs)} fresh={len(fresh)}", flush=True)
    if not fresh:
        return

    # 新仓
    idx = json.load(open(INDEX)) if os.path.exists(INDEX) else {}
    n_repo = 0
    for c in fresh:
        for m in re.findall(
            r"github\.com/([A-Za-z0-9_.\-]+/[A-Za-z0-9_.\-]+)", json.dumps(c)
        ):
            r = m.strip("/").lower()
            if r.count("/") == 1 and not any(
                x in r for x in [".svg", ".png", "issues", "pull", "blob", "tree"]
            ):
                if r not in idx:
                    idx[r] = {
                        "boards": ["openchamber"],
                        "stars": "",
                        "gained": "",
                        "desc": "",
                    }
                    n_repo += 1
    json.dump(idx, open(INDEX, "w"), indent=1, ensure_ascii=False)

    # 新卡记忆（按 views 取 top，防噪音）
    fresh_sorted = sorted(fresh, key=lambda c: -(c.get("v", 0) or 0))[: args.top]
    out_path = os.path.join(args.repo, "models/training/jev_builds.jsonl")
    n_card = 0
    with open(out_path, "a", encoding="utf-8") as f:
        for c in fresh_sorted:
            title = (c.get("t") or "")[:120]
            body = (c.get("x") or "")[:600]
            if not title and not body:
                continue
            f.write(
                json.dumps(
                    {
                        "url": c.get("url", ""),
                        "title": f"[jev-build] {title}",
                        "content": (
                            f"{title}\n{body}\n(category={c.get('cat')}, "
                            f"views={c.get('v')}, by=@{c.get('sn')})"
                        )[:800],
                        "domain": "jev-builds",
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )
            n_card += 1
    json.dump(sorted(seen | {str(c.get("id")) for c in cs}), open(SEEN, "w"))
    print(f"[oc] new repos={n_repo} new card memories={n_card}", flush=True)


if __name__ == "__main__":
    main()
