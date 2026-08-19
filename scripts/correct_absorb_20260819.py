#!/usr/bin/env python3
"""吸收批次 2026-08-19 能力映射人工校正 (7 条)。

关键词启发式误映射校正 (见 notes/absorption-20260819-unified-matrix.md):
  chinese-poetry: NT-SHIELD/audit → NT-MEMORY/retrieve (语料资产 + provenance 纠错)
  Pake:           NT-SHIELD/audit → NT-IO/invoke (桌面打包/CLI 契约)
  web-to-app:     NT-SHIELD/audit → NT-SHIELD/proxy (指纹伪装) + NT-ACT
  TRex:           NT-SHIELD/audit → NT-IO/invoke (OCR/URL-scheme 自动化)
  potpie:         NT-SHIELD/audit → NT-MEMORY/search (上下文图/检索)
  bettercap:      NT-SHIELD/verify → 保留 (分层代理正确, 不改)
  gnap:           NT-IO/delegate → NT-ACT/orchestrate (agent 协调)

写回仍走 R-P97: Rust CLI `update-node-metadata` (单一事实源, merge 语义)。
"""
import json
import os
import subprocess
import sys
import time

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
RUST_BIN = os.environ.get("NEOTRIX_EXPERIENCE_BIN", "neotrix-experience")

CORRECTIONS = [
    ("batch_1787130370_42ac62ce", "NT-MEMORY", "retrieve", "manual_correction: chinese-poetry 为中文语料资产+OCR 纠错, 属检索/记忆而非审计"),
    ("batch_1787130370_25b40aa5", "NT-IO", "invoke", "manual_correction: Pake 一键打包桌面应用/CLI 契约, 属界面执行而非审计"),
    ("batch_1787130370_43e1014c", "NT-SHIELD", "proxy", "manual_correction: web-to-app 50+指纹伪装/反检测, 属代理伪装层 (proxy)"),
    ("batch_1787130370_ca60cfce", "NT-IO", "invoke", "manual_correction: TRex 屏幕 OCR + URL-scheme 自动化, 属界面调用而非审计"),
    ("batch_1787130370_e3b0653e", "NT-MEMORY", "search", "manual_correction: potpie 代码上下文图/跨文件检索, 属检索而非审计"),
    ("batch_1787130370_fd714523", "NT-ACT", "orchestrate", "manual_correction: gnap git-as-transport 多 agent 协调, 属行动编排而非委托"),
]


def main():
    dry_run = "--dry-run" in sys.argv
    import sqlite3
    conn = sqlite3.connect(KB_PATH)
    now = int(time.time())
    updates = []
    for nid, branch, capability, evidence in CORRECTIONS:
        row = conn.execute(
            "SELECT title, json_extract(metadata,'$.absorbed_capability') FROM nodes WHERE id=?", (nid,)
        ).fetchone()
        if not row:
            print(f"  ✗ {nid} 不在库, 跳过", flush=True)
            continue
        old = row[1] or "{}"
        updates.append({
            "node_id": nid,
            "patch": {
                "absorbed_capability": {
                    "branch": branch,
                    "capability": capability,
                    "evidence": evidence,
                    "mapped_at": now,
                }
            }
        })
        print(f"  {row[0]:<16} {old[:40]:<45} → {branch}/{capability}", flush=True)
    conn.close()

    if dry_run:
        print(f"[dry-run] 将提交 {len(updates)} 条校正", flush=True)
        return

    tmp = "/tmp/nt_correction_20260819.json"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(updates, f, ensure_ascii=False)
    cmd = [RUST_BIN, "update-node-metadata", tmp]
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    out = (r.stdout or "") + (r.stderr or "")
    for line in out.splitlines():
        if "update-node-metadata" in line or "updated" in line:
            print(f"  {line}", flush=True)
    print(out[-2000:], flush=True)


if __name__ == "__main__":
    main()