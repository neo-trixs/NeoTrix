#!/usr/bin/env python3
"""D2 复用率周报（只读 KB → sessions/report-reuse.md）.

用法: python3 scripts/ops/nt_reuse_report.py
口径见 docs/plans/2026-09-24-d2-reuse-metric.md §3。
首周基线：helper_use 为空 → reused=0（未知→已知第一步，如实标“未插桩”）。
"""
import datetime
import sqlite3
import sys

KB = "/Users/neo/.neotrix/knowledge.db"
OUT = "sessions/report-reuse.md"


def main():
    c = sqlite3.connect(f"file:{KB}?mode=ro", uri=True)
    q = lambda t: c.execute(f"SELECT COUNT(*) FROM {t}").fetchone()[0]
    causal, procedural, exp = q("causal_rules"), q("procedural_memory"), q("experience")
    uses = c.execute(
        "SELECT key, value FROM kv_store WHERE namespace='helper_use'"
    ).fetchall()
    import json
    import time
    now = time.time()
    reused = 0
    for _, v in uses:
        try:
            u = json.loads(v)
            if now - u.get("last_used", 0) < 7 * 86400:
                reused += 1
        except Exception:
            pass
    created = causal + procedural
    rate = (reused / created) if created else 0.0
    today = datetime.date.today().isoformat()
    status = "未插桩（bg loop 常驻后打点）" if not uses else "已打点"
    body = f"""# 复用率周报 — {today}

- helpers 打点数: {len(uses)}（{status}）
- 7 天复用数: {reused}
- 持久 helper 存量: {created}（causal_rules={causal} + procedural={procedural}）
- reuse_rate: {rate:.3f}
- experience 池: {exp}

> 口径：reused/created（首周分母为存量，见 D2 设计 §3 注记）。
"""
    open(OUT, "w").write(body)
    print(f"wrote {OUT} rate={rate:.3f} n={len(uses)}")


if __name__ == "__main__":
    sys.exit(main())
