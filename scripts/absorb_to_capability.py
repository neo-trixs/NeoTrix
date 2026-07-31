#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NeoTrix 吸收→能力树映射器 (Absorb-to-Capability Mapper)
============================================================
把 batch_% 吸收节点 (repository/paper/article) 映射到 36 原子能力 + 7 域 BranchKind,
写入 CapabilityBranch.absorbed_capabilities (Cycle 121 字段), 产出覆盖率报告。

36 原子能力 (Cycle 121) × 9 层:
  PERCEIVE   : retrieve/search/observe/receive
  UNDERSTAND : detect/classify/measure/predict/compare/discover
  REASON     : plan/decompose/critique/explain
  MODEL      : state/transition/attribute/ground/simulate
  SYNTHESIZE : generate/transform/integrate
  EXECUTE    : execute/mutate/send
  VERIFY     : verify/checkpoint/rollback/constrain/audit
  REMEMBER   : persist/recall
  COORDINATE : delegate/synchronize/invoke/inquire

7 域:
  NT-CORE, NT-MIND, NT-MEMORY, NT-WORLD, NT-ACT, NT-SHIELD, NT-IO

用法:
  python3 scripts/absorb_to_capability.py --dry-run   # 预览映射
  python3 scripts/absorb_to_capability.py --apply     # 写入 KB
  python3 scripts/absorb_to_capability.py --report    # 只输出覆盖率报告
"""

import argparse
import json
import os
import re
import sqlite3
import sys
import time

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")

# 7 域 → 主属能力 (36 原子能力 Cycle 121)
BRANCH_CAPABILITIES = {
    "NT-CORE":   ["detect", "classify", "measure", "predict", "compare", "discover",
                  "plan", "decompose", "critique", "explain"],
    "NT-MIND":   ["generate", "transform", "integrate", "plan", "decompose"],
    "NT-MEMORY": ["state", "transition", "attribute", "ground", "simulate",
                  "persist", "recall"],
    "NT-WORLD":  ["retrieve", "search", "observe", "receive"],
    "NT-ACT":    ["execute", "mutate", "send"],
    "NT-SHIELD": ["verify", "checkpoint", "rollback", "constrain", "audit"],
    "NT-IO":     ["delegate", "synchronize", "invoke", "inquire"],
}
ALL_CAPABILITIES = sorted({c for caps in BRANCH_CAPABILITIES.values() for c in caps})

# 关键词 → (域, 能力) 规则表
KEYWORD_RULES = [
    # 爬虫/搜索/抓取
    (re.compile(r'crawl|scrap|fetcher|spider|crawler|browser|harvest|extract_web', re.I), "NT-WORLD", "retrieve"),
    (re.compile(r'search|retriev|index|semantic_search|rag|vector', re.I), "NT-WORLD", "search"),
    (re.compile(r'osint|recon|reconnaissance|subdomain|whois|dns_lookup|port_scan', re.I), "NT-WORLD", "observe"),
    # 理解/分析
    (re.compile(r'analyz|understand|classif|detect|cluster|topic_model|ner\b|segment', re.I), "NT-CORE", "detect"),
    (re.compile(r'metric|measure|score|benchmark|eval|quantif|statistic', re.I), "NT-CORE", "measure"),
    (re.compile(r'predict|forecast|forecast|trend|market', re.I), "NT-CORE", "predict"),
    (re.compile(r'plan|roadmap|scheduler|task_plan|goal', re.I), "NT-CORE", "plan"),
    # 推理
    (re.compile(r'reason|logic|infer|deduc|inference|chain_of_thought|debate|critique', re.I), "NT-CORE", "critique"),
    (re.compile(r'explain|interpret|insight|attribution|xai\b', re.I), "NT-CORE", "explain"),
    # 生成/合成
    (re.compile(r'generate|llm|gpt|model|prompt|text_gen|completion|image_gen|video_gen', re.I), "NT-MIND", "generate"),
    (re.compile(r'transform|translat|convert|summariz|rewrite|polish', re.I), "NT-MIND", "transform"),
    (re.compile(r'integrat|orchestrat|pipeline|workflow|compose|plugin', re.I), "NT-MIND", "integrate"),
    # 记忆/模型
    (re.compile(r'memory|remember|recall|store|persist|knowledge_base|kb\b|database|db\b', re.I), "NT-MEMORY", "recall"),
    (re.compile(r'model|simulat|world_model|environment|state_machine', re.I), "NT-MEMORY", "simulate"),
    # 执行/工具
    (re.compile(r'execut|tool|action|automation|script|cli\b|command|terminal|shell', re.I), "NT-ACT", "execute"),
    (re.compile(r'\bsdk\b|\bclient library\b|\blibrary\b|rest api wrapper|api wrapper|sdk for', re.I), "NT-ACT", "send"),
    (re.compile(r'\bmcp (server|client|protocol)\b|mcp-|/mcp\b', re.I), "NT-ACT", "send"),
    (re.compile(r'\bwebhook|notification|messaging|push\b|telegram|slack|discord|wechat', re.I), "NT-ACT", "send"),
    # 安全/验证
    (re.compile(r'security|vuln|audit|scan|pen_test|pentest|exploit|firewall|shield|protect', re.I), "NT-SHIELD", "audit"),
    (re.compile(r'verify|test|validate|check|quality|assert|lint', re.I), "NT-SHIELD", "verify"),
    # 界面/通信
    (re.compile(r'ui\b|ux\b|interface|frontend|design|dashboard|visual|component', re.I), "NT-IO", "invoke"),
    (re.compile(r'communicat|chat|message|socket|stream|real_time|notify|webhook', re.I), "NT-IO", "synchronize"),
    (re.compile(r'agent|multi_agent|delegate|subagent|swarm|coordinator|router', re.I), "NT-IO", "delegate"),
    (re.compile(r'provider|gateway|model_router|llm_api|auth|login|sso|oauth', re.I), "NT-IO", "inquire"),
]

# 代码库已有节点名 → 能力树 (NT- 域) — 为 repository 节点提供确定性映射
KNOWN_REPOS = {
    "mattpocock/skills": ("NT-CORE", "plan"),       # shared language + domain skills
    "anthropics/skills": ("NT-CORE", "plan"),
    "google/skills": ("NT-CORE", "plan"),
    "claude-code": ("NT-ACT", "execute"),
    "openai/codex": ("NT-ACT", "execute"),
    "tauri": ("NT-IO", "invoke"),
    "camoufox": ("NT-SHIELD", "constrain"),
    "firecrawl": ("NT-WORLD", "retrieve"),
    "crawl4ai": ("NT-WORLD", "retrieve"),
    "OpenHands": ("NT-ACT", "execute"),
    "AutoAgent": ("NT-ACT", "execute"),
    "khoj": ("NT-MEMORY", "recall"),
    "librechat": ("NT-IO", "synchronize"),
    "langfuse": ("NT-SHIELD", "verify"),
    "flowise": ("NT-MIND", "integrate"),
    "markitdown": ("NT-MIND", "transform"),
    "maigret": ("NT-WORLD", "observe"),
    "MediaCrawler": ("NT-WORLD", "retrieve"),
    "mem0": ("NT-MEMORY", "recall"),
    "LightMem": ("NT-MEMORY", "recall"),
    "SimpleMem": ("NT-MEMORY", "recall"),
    "croc": ("NT-ACT", "send"),
    "kimi": ("NT-MIND", "generate"),
    "DeepSeek-V3": ("NT-MIND", "generate"),
    "UI-TARS": ("NT-WORLD", "observe"),
    "OmniParser": ("NT-WORLD", "observe"),
    "docling": ("NT-MIND", "transform"),
    "mineru": ("NT-MIND", "transform"),
    "GFPGAN": ("NT-MIND", "transform"),
    "exo": ("NT-IO", "inquire"),
    "ollama": ("NT-IO", "inquire"),
    "OpenManus": ("NT-ACT", "execute"),
    "Fabric": ("NT-MIND", "integrate"),
    "fabric": ("NT-MIND", "integrate"),
    "awesome-llm-apps": ("NT-MIND", "integrate"),
    "OpenResearcher": ("NT-CORE", "explain"),
    "FinanceDatabase": ("NT-CORE", "predict"),
    "supavec": ("NT-IO", "invoke"),
    "context7": ("NT-MEMORY", "recall"),
    "unstract": ("NT-MEMORY", "recall"),
    "kotaemon": ("NT-MEMORY", "recall"),
    "Serena": ("NT-ACT", "execute"),
    "jan": ("NT-IO", "inquire"),
    "chatbox": ("NT-IO", "synchronize"),
    "copilotkit": ("NT-MIND", "generate"),
    "Remotion": ("NT-MIND", "generate"),
    "hyperframes": ("NT-MIND", "generate"),
    "remotion": ("NT-MIND", "generate"),
    "livetalking": ("NT-MIND", "generate"),
    "pipecat": ("NT-IO", "synchronize"),
    "meshflow": ("NT-MIND", "generate"),
    "MeshFlow": ("NT-MIND", "generate"),
    "kroko": ("NT-MIND", "generate"),
    "graphify": ("NT-MEMORY", "search"),
    "Graphify": ("NT-MEMORY", "search"),
    "khoj-ai": ("NT-MEMORY", "recall"),
}


def normalize_repo_title(title):
    """'Karpathy AutoResearch' 或 'GitHub - owner/repo: desc' → owner/repo"""
    t = re.sub(r'^GitHub\s*-\s*', '', title)
    t = t.split(':')[0].strip()
    return t


def map_node(node_type, title, content, url):
    """返回 (branch, capability, evidence) 或 None"""
    owner_repo = None
    if node_type == 'repository':
        owner_repo = normalize_repo_title(title)
        low = owner_repo.lower()
        # 先查 KNOWN_REPOS (确定性)
        for k, (br, cap) in KNOWN_REPOS.items():
            if k.lower() in low or low.endswith(k.lower()):
                return br, cap, f'known_repo:{k}'
    elif node_type == 'paper':
        low = title.lower()
    else:
        low = title.lower()

    blob = ' '.join([title, content[:1200]])
    best = None
    best_hits = 0
    for pat, br, cap in KEYWORD_RULES:
        hits = len(pat.findall(blob))
        # title 命中权重 ×3 (title 比 README 正文更具判别力)
        title_hits = len(pat.findall(title))
        score = title_hits * 3 + hits
        if score > best_hits:
            best_hits = score
            best = (br, cap)
    if best:
        return best[0], best[1], f'keyword_hits:{best_hits}'
    # 兜底: repository → NT-WORLD.retrieve, paper → NT-CORE.critique
    if node_type == 'repository':
        return 'NT-WORLD', 'retrieve', 'fallback:repo'
    if node_type == 'paper':
        return 'NT-CORE', 'critique', 'fallback:paper'
    return 'NT-CORE', 'discover', 'fallback:article'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--apply', action='store_true', help='写入 KB (absorbed_capabilities 字段)')
    ap.add_argument('--report', action='store_true', help='只输出覆盖率报告')
    args = ap.parse_args()

    conn = sqlite3.connect(KB_PATH)
    rows = conn.execute("""SELECT id, node_type, title, content, url FROM nodes
                           WHERE id LIKE 'batch_%'""").fetchall()
    print(f'[mapping] {len(rows)} batch nodes', flush=True)

    mapped = {}
    per_branch = {}
    per_cap = {}
    unmapped = []
    for nid, node_type, title, content, url in rows:
        res = map_node(node_type, title, content, url)
        if res is None:
            unmapped.append((nid, title))
            continue
        branch, cap, ev = res
        mapped[nid] = {'branch': branch, 'capability': cap, 'evidence': ev,
                       'node_type': node_type, 'title': title[:60], 'url': url}
        per_branch.setdefault(branch, []).append(cap)
        per_cap[cap] = per_cap.get(cap, 0) + 1

    if args.apply:
        now = int(time.time())
        for nid, m in mapped.items():
            try:
                cur = conn.execute("SELECT metadata FROM nodes WHERE id=?", (nid,)).fetchone()
                meta = {}
                if cur and cur[0]:
                    try:
                        meta = json.loads(cur[0])
                    except json.JSONDecodeError:
                        meta = {}
                meta['absorbed_capability'] = {'branch': m['branch'],
                                               'capability': m['capability'],
                                               'evidence': m['evidence'],
                                               'mapped_at': now}
                conn.execute("UPDATE nodes SET metadata=? WHERE id=?",
                             (json.dumps(meta, ensure_ascii=False), nid))
            except sqlite3.Error as e:
                print(f'  ✗ {nid}: {e}', flush=True)
        conn.commit()
        print(f'[mapping] wrote {len(mapped)} capability mappings to KB', flush=True)

    # ── 覆盖率报告 ──
    print(f'\n=== 覆盖率报告 ===')
    print(f'映射成功: {len(mapped)}/{len(rows)} ({100*len(mapped)/max(1,len(rows)):.1f}%)')
    print(f'未映射:   {len(unmapped)}')
    print(f'\n--- 7 域分布 ---')
    for br, caps in sorted(per_branch.items(), key=lambda x: -len(x[1])):
        uniq = len(set(caps))
        print(f'  {br:<12} {len(caps):>4} 次  ({uniq} 能力)')
    print(f'\n--- 36 能力分布 (top 12) ---')
    for cap, cnt in sorted(per_cap.items(), key=lambda x: -x[1])[:12]:
        print(f'  {cap:<14} {cnt:>4}')
    print(f'\n--- 未映射节点 ---')
    for nid, t in unmapped[:15]:
        print(f'  {nid}  {t[:60]}')

    conn.close()


if __name__ == '__main__':
    main()
