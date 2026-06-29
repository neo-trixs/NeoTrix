#!/usr/bin/env python3
"""NeoTrix 记忆巩固管线 (Memory Consolidation Pipeline)

超越人脑记忆的三层架构: 
- 人脑: 遗忘曲线被动衰减, 近似回忆
- 本管线: 确定性压缩 + 自信度追踪 + 自动蒸馏 + 无损存档

Usage:
  python3 scripts/compress_agents.py              # 全管线执行
  python3 scripts/compress_agents.py --verify     # 一致性验证
  python3 scripts/compress_agents.py --distill    # 仅蒸馏会话日志
  python3 scripts/compress_agents.py --prune      # 仅修剪低自信度规则
"""

import re, os, sys, json
from pathlib import Path
from datetime import date

ROOT = Path(__file__).parent.parent
AGENTS = ROOT / "AGENTS.md"
SELF   = ROOT / "SELF.md"
RULES  = ROOT / "RULES.md"
ARCHIVE = ROOT / "archive"
SESSIONS = ARCHIVE / "sessions"
CONSOLIDATED = ARCHIVE / "consolidated"

RULE_TRUNCATE = 100  # max chars for rule text
MIN_CONF_KEEP = 0.30  # below this → archive or delete

def extract_sessions(text):
    sessions = []
    for m in re.finditer(r'(## 会话日志: .*?)(?=\n## |\Z)', text, re.DOTALL):
        content = m.group(1).strip()
        title = content.split("\n")[0].replace("## 会话日志: ", "").strip()
        key_event = ""
        for line in content.split("\n"):
            if "**关键事件**" in line:
                key_event = line.replace("**关键事件**: ", "").strip()[:200]
                break
        sessions.append({"title": title, "content": content, "key": key_event})
    return sessions

def extract_rules(text):
    tree_start = text.find("## 经验树（Experience Tree）")
    if tree_start < 0:
        tree_start = text.find("## 经验树")
    if tree_start < 0:
        return []
    tree_text = text[tree_start:]
    sections = re.split(r'(?=^### )', tree_text, flags=re.MULTILINE)
    rules = []
    for sec in sections:
        bm = re.match(r'^### 分支 ([^—\n]+)(?: — | )(.*)', sec, re.MULTILINE)
        if not bm:
            continue
        branch_id = bm.group(1).strip()
        branch_name = bm.group(2).strip()[:50]
        parts = re.split(r'(?=^#### )', sec, flags=re.MULTILINE)
        for part in parts:
            if not part.startswith("#### "):
                continue
            nm = re.match(r'^#### ([A-Z0-9.IVX]+(?:\.\d+)?)\s+(.+?)$', part, re.MULTILINE)
            if not nm:
                continue
            node_id = nm.group(1).strip()
            node_name = nm.group(2).strip()[:30]
            body = part[nm.end():]
            cm = re.search(r'\*\*conf\*\*:\s*([0-9.]+)', body)
            conf = cm.group(1) if cm else "?"
            rm = re.search(r'\*\*规则\*\*:\s*(.+?)(?:\n\*{2}|\n\n|\Z)', body, re.DOTALL)
            rule = ""
            if rm:
                rule = rm.group(1).strip().replace("\n", " ").replace("**", "")[:RULE_TRUNCATE]
            em = re.search(r'\*\*演化链\*\*:\s*(.+?)$', body, re.MULTILINE)
            evo = em.group(1).strip()[:30] if em else ""
            rules.append({
                "id": node_id, "conf": conf, "name": node_name,
                "rule": rule, "evo": evo,
                "branch": f"{branch_id} — {branch_name}"
            })
    return rules

def generate_rules_md(rules):
    lines = [
        "# RULES: NeoTrix 规则树",
        f"# total: {len(rules)} nodes | gen: {date.today()}",
        "# fmt: id|conf|name|rule|evo",
        ""
    ]
    current_branch = ""
    for r in rules:
        if r["branch"] != current_branch:
            current_branch = r["branch"]
            lines.append(f"# {current_branch}")
        lines.append(f'{r["id"]}|{r["conf"]}|{r["name"]}|{r["rule"]}|{r["evo"]}')
    return lines

def distill_session(session_text):
    """将原始会话日志蒸馏为 3-5 行摘要."""
    lines = session_text.split("\n")
    title = lines[0] if lines else ""
    key_event = ""
    summary = []
    in_table = False
    for line in lines:
        if "**关键事件**" in line:
            key_event = line.replace("**关键事件**: ", "").strip()[:200]
        if "| 领域 |" in line and "状态" in line:
            in_table = True
            continue
        if in_table and "|" in line and "---" not in line:
            parts = [p.strip() for p in line.split("|") if p.strip()]
            if len(parts) >= 3:
                summary.append(f"{parts[1]}:{parts[-1]}")
    result = f"{title}\n\nKey: {key_event}\n" if key_event else f"{title}\n\n"
    if summary:
        result += f"Tasks({len(summary)}): {', '.join(summary[:6])}\n"
    return result

def prune_low_confidence(rules):
    """修剪低自信度规则. 返回 (keep, archived)."""
    keep = []
    archived = []
    for r in rules:
        try:
            conf = float(r["conf"]) if r["conf"] != "?" else 0.0
        except ValueError:
            conf = 0.0
        if conf >= MIN_CONF_KEEP:
            keep.append(r)
        else:
            archived.append(r)
    return keep, archived

def verify_consistency():
    """验证存储架构一致性."""
    issues = []
    
    # Check SELF.md exists
    if not SELF.exists():
        issues.append("MISSING: SELF.md")
    if not RULES.exists():
        issues.append("MISSING: RULES.md")
    
    # Count nodes in RULES.md
    if RULES.exists():
        rule_lines = RULES.read_text().split("\n")
        node_count = sum(1 for l in rule_lines if l and not l.startswith("#"))
        if node_count < 200:
            issues.append(f"LOW NODE COUNT: {node_count} (expected 237)")
    
    # Check session files
    if SESSIONS.exists():
        session_files = list(SESSIONS.glob("*.md"))
        if len(session_files) < 60:
            issues.append(f"LOW SESSION COUNT: {len(session_files)} (expected 65)")
        # Check they're distilled (small)
        large = [f for f in session_files if f.stat().st_size > 5000]
        if large:
            issues.append(f"UNDISTILLED SESSIONS: {len(large)} files >5KB")
    
    # Check consolidated exists
    for fname in ["NARRATIVE.md", "SALIENCE.md", "INDEX.md", "TIMELINE.md", "GRAPH.md", "CONFIDENCE.md"]:
        if not (CONSOLIDATED / fname).exists():
            issues.append(f"MISSING: archive/consolidated/{fname}")
    
    return issues

def main():
    import argparse
    parser = argparse.ArgumentParser(description="NeoTrix Memory Consolidation")
    parser.add_argument("--verify", action="store_true", help="Verify consistency")
    parser.add_argument("--distill", action="store_true", help="Distill sessions only")
    parser.add_argument("--prune", action="store_true", help="Prune low-confidence rules")
    args = parser.parse_args()
    
    if args.verify:
        issues = verify_consistency()
        if issues:
            print("VERIFY FAILED:")
            for i in issues:
                print(f"  ❌ {i}")
            return 1
        else:
            print("✅ All consistency checks passed")
            return 0
    
    if not AGENTS.exists():
        print("No AGENTS.md found", file=sys.stderr)
        return 1
    
    text = AGENTS.read_text()
    rules = extract_rules(text)
    sessions = extract_sessions(text)
    
    if args.distill:
        SESSIONS.mkdir(parents=True, exist_ok=True)
        count = 0
        for s in sessions:
            safe_title = s["title"].replace(" ", "_").replace("/", "-")[:60]
            fname = f"{s['title'][:10]}_{safe_title}.md"
            dist = distill_session(s["content"])
            (SESSIONS / fname).write_text(dist)
            count += 1
        print(f"Distilled {count} sessions to {SESSIONS}")
        return 0
    
    if args.prune:
        kept, archived = prune_low_confidence(rules)
        print(f"Prune: {len(kept)} kept, {len(archived)} archived")
        if archived:
            archive_path = ARCHIVE / "pruned_rules.md"
            with open(archive_path, "w") as f:
                for r in archived:
                    f.write(f'{r["id"]}|{r["conf"]}|{r["name"]}|{r["rule"]}|{r["branch"]}\n')
            print(f"Archived to {archive_path}")
        # Write kept rules to RULES.md
        RULES.write_text("\n".join(generate_rules_md(kept)))
        return 0
    
    # Full pipeline
    rules_lines = generate_rules_md(rules)
    SESSIONS.mkdir(parents=True, exist_ok=True)
    CONSOLIDATED.mkdir(parents=True, exist_ok=True)
    
    for s in sessions:
        safe_title = s["title"].replace(" ", "_").replace("/", "-")[:60]
        fname = f"{s['title'][:10]}_{safe_title}.md"
        dist = distill_session(s["content"])
        (SESSIONS / fname).write_text(dist)
    
    RULES.write_text("\n".join(rules_lines))
    print(f"✅ Full consolidation complete")
    print(f"   Rules: {len(rules_lines)-2} lines from {len(rules)} nodes")
    print(f"   Sessions: {len(sessions)} distilled")
    print(f"   SELF.md: {SELF.stat().st_size} bytes" if SELF.exists() else "   SELF.md: MISSING")
    
    issues = verify_consistency()
    if issues:
        print(f"   ⚠️ {len(issues)} issues: {', '.join(issues[:3])}")
    else:
        print(f"   ✅ All consistency checks passed")

if __name__ == "__main__":
    main()
