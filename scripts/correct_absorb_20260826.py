#!/usr/bin/env python3
"""吸收批次 2026-08-26 能力映射人工校正 (31 条)。

关键词启发式误映射校正 — 每条均经 README/API description 接地复核 (C5 evidence-first)。
依据先例: Cycle 1192/1201/1208 校正门 · camoufox→SHIELD/constrain (伪装) ·
codex→ACT/execute · 文档转换→MIND/transform · 上下文管理→MEMORY/transform。

写回走 R-P97: Rust CLI `update-node-metadata` (单一事实源, merge 语义)。
"""
import json
import os
import subprocess
import sys
import time

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
RUST_BIN = os.environ.get("NEOTRIX_EXPERIENCE_BIN", "neotrix-experience")

# (URL 子串, 新 branch, 新 capability, 证据)
CORRECTIONS = [
    ("nativePDF-structurer", "NT-MIND", "transform",
     "manual_correction: 零OCR大体量PDF结构化=文档转换 (Cycle1208 mdflux 先例: 文档转换→MIND/transform)"),
    ("poka-yoke", "NT-SHIELD", "constrain",
     "manual_correction: agent 编辑防错约束披露门 'what its fix makes impossible' = mistake-proofing constrain (camoufox→constrain 先例域)"),
    ("viperrcrypto/Siftly", "NT-MEMORY", "recall",
     "manual_correction: 自托管 X 书签 AI 组织+语义检索 = 个人记忆库 recall"),
    ("ix-infrastructure/Ix", "NT-MEMORY", "recall",
     "manual_correction: 代码库上下文图 'virtual cartographer' = 图检索 (dev-rules BlastRadiusIndex→NT-MEMORY KB 先例)"),
    ("easy_tdx", "NT-WORLD", "retrieve",
     "manual_correction: 通达信毫秒级行情数据摄取 = 金融数据 retrieve (crawl4ai→WORLD/retrieve 先例)"),
    ("freellmapi", "NT-IO", "delegate",
     "manual_correction: 34 免费 LLM provider 聚合 + 路由 failover = NT-IO LLM providers 域 delegate"),
    ("Yu9191/wloc", "NT-SHIELD", "constrain",
     "manual_correction: Apple WLOC MITM 定位伪造 = 位置指纹伪装层 (指纹伪装→proxy/constrain 先例)"),
    ("AI-Crash-Course", "NT-IO", "inquire",
     "manual_correction: 2周 AI 前沿学习路径 = 教育解释面 (ed/tutor→NT-IO 收编先例 Edu-灯)"),
    ("Toonflow-app", "NT-MIND", "generate",
     "manual_correction: 小说→动画短剧 AI 生成流水线 (编剧/分镜/视频生成, API description 接地)"),
    ("an2tha/onerep", "NT-WORLD", "observe",
     "manual_correction: 开源健身 OS 训练/营养/恢复数据观察 = 身体指标 observe"),
    ("agentforce314/clawcodex", "NT-ACT", "execute",
     "manual_correction: Claude Code Python 重建 = codex 类编码执行 agent (KNOWN_REPOS openai/codex→ACT/execute 先例)"),
    ("adaptive-zsh-completions", "NT-IO", "inquire",
     "manual_correction: 学习式 shell 补全 = CLI 界面交互 inquire (CLI 属 NT-IO)"),
    ("world-intel-mcp", "NT-WORLD", "observe",
     "manual_correction: 120 MCP 工具全球情报采集 + Qdrant 语义搜索 = 情报观察 (intel-watch 同类)"),
    ("COG-second-brain", "NT-MEMORY", "recall",
     "manual_correction: .md 第二大脑自进化记忆系统 = knowledge recall"),
    ("MengTo/sketchbook", "NT-MIND", "generate",
     "manual_correction: 单文件交互创意前端 (纸张物理效果) = 创意生成"),
    ("BraxisAI/braxis-blueprint", "NT-ACT", "execute",
     "manual_correction: 免费档 140+ 自主 agent 编排运营蓝图 = 行动执行编排"),
    ("odysseus-dev/odysseus", "NT-IO", "synchronize",
     "manual_correction: 自托管 AI workspace 集成 chat/email/calendar/notes = 多服务同步界面层"),
    ("tt-a1i/archify", "NT-MIND", "transform",
     "manual_correction: 代码库/描述→交互式系统图 = 架构可视化变换 (fireworks-tech-graph 同类)"),
    ("kyegomez/OpenMythos", "NT-MIND", "transform",
     "manual_correction: 从公开文献第一性原理重建 Claude Mythos 架构 = 架构蒸馏 transform"),
    ("YuJunZhiXue/dsh-purge", "NT-SHIELD", "observe",
     "manual_correction: jailbreak 红队威胁参考 ('no more refusals', API desc 接地) = 攻击面观察"),
    ("arxiv.org/abs/2608.19741", "NT-SHIELD", "verify",
     "manual_correction: Thinkingbox stateful 业务 agent 沙箱基准 = 沙箱验证 (Egress Policy→nt_shield_sandbox 先例)"),
    ("alphaxiv.org/abs/2608.23552", "NT-MIND", "transform",
     "manual_correction: Prime Agent 自改进 RLM harness = SEAL 同类自进化循环"),
    ("paperswithcode.co/paper/2608.23552", "NT-MIND", "transform",
     "manual_correction: Prime Agent 自改进 RLM harness (重复源同映射) = SEAL 同类自进化循环"),
    ("arxiv.org/abs/2608.19760", "NT-SHIELD", "audit",
     "manual_correction: 无真值步级信用分配审计 (title 含 Auditing) = agent 行为审计"),
    ("arxiv.org/abs/2608.22752", "NT-MEMORY", "compress",
     "manual_correction: 长程 agent 记忆压缩断崖 Compaction Cliff = compress (上下文管理→MEMORY/transform 先例, 词表精确命中)"),
    ("arxiv.org/abs/2608.20256", "NT-CORE", "route",
     "manual_correction: 测试时算力自适应分配 Learning When to Think = GWT attention routing 同类 (route 在 CORE 词表)"),
    ("arxiv.org/abs/2608.20845", "NT-MEMORY", "persist",
     "manual_correction: RAG ingest 时索引编译优于查询时解释 = KB 入库索引策略 persist"),
    ("arxiv.org/abs/2601.23265", "NT-MIND", "generate",
     "manual_correction: PaperBanana AI 科学家学术插图自动化 = 视觉生成"),
    ("paperswithcode.co/paper/2608.16812", "NT-MIND", "generate",
     "manual_correction: 概念缩放+密集监督图像编辑 = 视觉生成编辑 (消费点 nt_io_multimodal_transform)"),
    ("eli5-org/eli5", "NT-CORE", "explain",
     "manual_correction: ML 分类器预测解释/调试库 = explainability (explain 在 CORE 词表精确命中)"),
    ("garrytan/gstack", "NT-ACT", "execute",
     "manual_correction: Garry Tan 23 角色 Claude Code 工具集 (CEO/设计/工程经理/QA) = 编码工作流行动执行"),
    ("yibie/awesome-autoresearch", "NT-MIND", "integrate",
     "manual_correction: 自动研究循环方法精选集 = SEAL pipeline 同类知识整合"),
]


def main():
    dry_run = "--dry-run" in sys.argv
    import sqlite3
    conn = sqlite3.connect(KB_PATH)
    now = int(time.time())
    updates = []
    corrected = skipped = unchanged = 0
    for url_sub, branch, capability, evidence in CORRECTIONS:
        rows = conn.execute(
            "SELECT id, title, json_extract(metadata,'$.absorbed_capability.branch'),"
            " json_extract(metadata,'$.absorbed_capability.capability')"
            " FROM nodes WHERE url LIKE ? AND id LIKE 'batch_%'", (f"%{url_sub}%",)
        ).fetchall()
        if not rows:
            print(f"  ✗ {url_sub}: 不在库, 跳过", flush=True)
            skipped += 1
            continue
        for nid, title, old_br, old_cap in rows:
            if old_br == branch and old_cap == capability:
                print(f"  ={title[:28]:<30} 已是 {branch}/{capability}", flush=True)
                unchanged += 1
                continue
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
            print(f"  ~{title[:28]:<30} {old_br}/{old_cap} → {branch}/{capability}", flush=True)
            corrected += 1
    conn.close()

    if dry_run:
        print(f"\n[dry-run] would_correct={corrected} unchanged={unchanged} missing={skipped}")
        return

    if updates:
        with open("/tmp/opencode/corr_20260826.json", "w") as f:
            json.dump(updates, f, ensure_ascii=False)
        result = subprocess.run(
            [RUST_BIN, "update-node-metadata", "/tmp/opencode/corr_20260826.json"],
            capture_output=True, text=True, timeout=60,
        )
        print(result.stdout[-800:])
        if result.returncode != 0:
            print(f"STDERR: {result.stderr[-500:]}", file=sys.stderr)
            sys.exit(1)

    # R-P16: re-read 校验持久化
    conn = sqlite3.connect(KB_PATH)
    verified = expected = 0
    for url_sub, branch, capability, _ in CORRECTIONS:
        row = conn.execute(
            "SELECT json_extract(metadata,'$.absorbed_capability.branch'),"
            " json_extract(metadata,'$.absorbed_capability.capability')"
            " FROM nodes WHERE url LIKE ? AND id LIKE 'batch_%'", (f"%{url_sub}%",)
        ).fetchone()
        if row:
            expected += 1
            if row[0] == branch and row[1] == capability:
                verified += 1
            else:
                print(f"  ✗ 未持久化: {url_sub} → {row[0]}/{row[1]} (期望 {branch}/{capability})", file=sys.stderr)
    conn.close()
    print(f"\n校正 {corrected} 条 | 持久化验证 {verified}/{expected} (R-P16)")


if __name__ == "__main__":
    main()
