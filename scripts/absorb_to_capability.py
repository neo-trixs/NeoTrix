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
import subprocess
import sys
import tempfile
import time

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")

# ── R-P97: 写回委托 Rust CLI (update-node-metadata) — 单一事实源 ──
RUST_BIN = os.environ.get("NEOTRIX_EXPERIENCE_BIN", "neotrix-experience")


def rust_update_node_metadata(updates, dry_run=False):
    """把 [{node_id, patch}] 列表交给 Rust CLI 批量 merge 写回。

    返回 (updated, missing)。Rust 侧读原 metadata → 合并 patch → 写回,
    保留既有字段 (如 topics/description), 仅覆盖 patch 声明的键。
    """
    if not updates:
        return (0, 0)
    tmp = tempfile.NamedTemporaryFile(
        mode='w', suffix='.json', prefix='nt_meta_', delete=False, encoding='utf-8')
    with tmp:
        json.dump(updates, tmp, ensure_ascii=False)
    cmd = [RUST_BIN, 'update-node-metadata', tmp.name]
    if dry_run:
        cmd.append('--dry-run')
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    except FileNotFoundError:
        print(f"  ✗ {RUST_BIN} 未找到 — 请先构建并安装 neotrix-experience", flush=True)
        return (0, 0)
    out = (r.stdout or '') + (r.stderr or '')
    for line in out.splitlines():
        if 'update-node-metadata' in line:
            print(f"  {line}", flush=True)
    m = re.search(r"(\d+) updated, (\d+) missing", out)
    if m:
        return (int(m.group(1)), int(m.group(2)))
    return (0, 0)


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

# ────────────────────────────────────────────────────────────────
# 本源溯源层 (Source Cores) — Cycle 161i
# 5 道之本源, 极简无细分。知识不是平行学科, 而是本源的分支脉络:
#   每个节点溯源到一个本源 (source_core) + 演化路径 (trace_path)。
#   交叉学科在本源深处自然交汇, 而非被文理分科割裂 (R-P42)。
# ────────────────────────────────────────────────────────────────
SOURCE_CORES = [
    # (本源名, 主属域, 判别关键词, 本源定义)
    ("E8",          "NT-CORE",   ["symmetr", "mathemat", "algebra", "geometry",
                                  "theorem", "axiom", "formal", "topolog", "calculus", "equation",
                                  "fractal", "invariant", "funct", "statistical mechan", "proof",
                                  "quantum", "thermodynam", "entrop", "relativit", "hamiltonian", "particle",
                                  "differential", "topolog", "set theor", "number theor", "homolog",
                                  "manifold", "tensor", "optimiz", "algorith", "complexity theor"],
                  "一切形式/结构/规律之源"),
    ("VSA",         "NT-MEMORY", ["memor", "semant", "represent", "vector", "embed", "symbol", "meaning",
                                  "concept", "knowledge base", "encod", "hypercub", "recall", "retrieve",
                                  "latent", "holographic", "state space", "distributed represent", "kb",
                                  "embedding", "knowledge graph", "hyperdimension", "ontolog", "semantic memory",
                                  "associative memor", "content-addressable", "episodic memor", "working memor",
                                  "dual-coding", "vector symbolic", "holographic represent"],
                  "一切概念/记忆/表示之源"),
    ("GWT",         "NT-CORE",   ["conscious", "consciousness", "percept", "aware", "cognition", "cognitiv",
                                  "global workspace", "integrat inform", "mind", "sentient", "binding",
                                  "focus", "thalamus", "metacognit", "introspect", "self-aware", "neurosci",
                                  "mental", "emotion", "brain", "neural activ", "cognitive architecture",
                                  "phenomenolog", "qualia", "self model", "cognitive model", "working memor",
                                  "cognitive science", "subjective experienc", "sense of self", "perception",
                                  "attention mechanism", "gwt", "workspace theor", "conscious experienc"],
                  "一切意识/感知/认知之源"),
    ("ConsciousnessTree", "NT-MIND", ["absorb", "distill", "crystalliz", "evolve", "self-improv", "learn",
                                      "adapt", "internaliz", "feedback", "growth", "self-heal", "recursion",
                                      "reflect", "experience", "pattern recognit", "intuition", "pruning",
                                      "meta-learn", "self-organiz", "self-evolv", "autonom", "curriculum"],
                  "一切元认知/吸收/演化之源"),
    ("Reality",     "NT-WORLD",  ["world", "world model", "agent", "act", "action", "interact", "environ",
                                  "sensor", "control", "tool", "execute", "robot", "simulat",
                                  "perceiv", "explore", "harvest", "crawl", "embodied", "real world",
                                  "physical", "device", "hardware", "deploy", "operate", "drone"],
                  "一切世界/感知/行动之源"),
]

# 本源兜底启发式 (FALLBACK HINTS): 无关键词命中时按标题线索词分源
# 本源哲学: 每节点终有所属本源; 线索词捕捉标题里的本源痕迹
FALLBACK_HINTS = [
    (["math", "phys", "theor", "scien", "logic", "philosoph", "quantum", "chem", "astron",
      "relativ", "biolog", "geolog", "crystal", "equat", "axiom", "proof", "formal"], "E8"),
    (["memor", "semantic", "represent", "concept", "knowledge", "intellig", "language", "symbol",
      "embed", "vector", "database", "graph", "word", "text"], "VSA"),
    (["conscious", "mind", "brain", "cogni", "percept", "psych", "emotion", "aware", "neuro",
      "attention", "mental", "dream"], "GWT"),
    (["learn", "evolv", "adapt", "growth", "self", "reflect", "experienc", "feedback",
      "develop", "train", "improv"], "ConsciousnessTree"),
    (["world", "action", "agent", "society", "polit", "econom", "hist", "culture", "art",
      "war", "power", "soci", "commun", "technolog", "engineer", "industr", "market", "law",
      "govern", "earth", "space", "human", "life"], "Reality"),
]


def fallback_source(title, node_type='article'):
    """兜底: 标题线索词分源。repository→Reality, paper→E8, 其余按线索词。
    无任何线索词 → Reality (世界知识大本营, 行动之源)。"""
    if node_type == 'repository':
        return "Reality", "NT-WORLD", ['tool']
    if node_type == 'paper':
        return "E8", "NT-CORE", ['theory']
    blob = (title or '').lower()
    for kws, core in FALLBACK_HINTS:
        if any(k in blob for k in kws):
            domain = next(d for c, d, _k, _z in SOURCE_CORES if c == core)
            return core, domain, [next(k for k in kws if k in blob)]
    return "Reality", "NT-WORLD", ['world']



#   paper        → 形式之源 (E8) 载体: 论文即形式化知识, 默认 E8
#   repository   → 行动之源 (Reality) 载体: 仓库即世界交互工具, 默认 Reality
#   article      → 中性
SOURCE_PRIOR = {
    'paper':       ('E8',      'NT-CORE',  0.35),   # 先验计入阈值
    'repository':  ('Reality', 'NT-WORLD', 0.0),    # 已由关键词判定
    'article':     (None,      None,       0.0),
}


def map_source_core(title, content, url, node_type='article'):
    """本源溯源: 返回 (source_core, primary_domain, trace_keywords) 或 (None, None, []).

    互斥判定: 取最高关键词命中数; 命中数相同取列表序靠前者 (确定性)。
    trace_path 由 top 命中的关键词片段构成 → 本源 → 分支 → 节点 的演化路径。
    paper 载体默认溯源 E8 (形式之源), 除非内容强命中 VSA/GWT/ConsciousnessTree。
    """
    blob = ' '.join([title, (content or '')[:2000]])
    best = None
    best_score = 0
    best_kws = []
    prior_core, prior_domain, prior_margin = SOURCE_PRIOR.get(node_type, (None, None, 0.0))
    for name, domain, kws, _def in SOURCE_CORES:
        hits = [(kw, len(re.findall(re.escape(kw), blob, re.I))) for kw in kws]
        score = sum(h for _, h in hits)
        if name == prior_core and score > 0:
            score += max(1, int(score * prior_margin)) + 2  # 先验权重
        if score > best_score:
            best_score = score
            best = (name, domain)
            best_kws = [kw for kw, h in sorted(hits, key=lambda x: -x[1])[:3] if h > 0]
    if best and best_score > 0:
        return best[0], best[1], best_kws
    return None, None, []

# 关键词 → (域, 能力) 规则表

# 关键词 → (域, 能力) 规则表
KEYWORD_RULES = [
    # 爬虫/搜索/抓取
    (re.compile(r'crawl|scrap|fetcher|spider|crawler|browser|harvest|extract_web', re.I), "NT-WORLD", "retrieve"),
    (re.compile(r'search|retriev|index|semantic_search|rag|vector', re.I), "NT-WORLD", "search"),
    (re.compile(r'osint|recon|reconnaissance|subdomain|whois|dns_lookup|port_scan', re.I), "NT-WORLD", "observe"),
    # 理解/分析
    (re.compile(r'explor|discover|survey|overview|invent|categoriz|taxonom|reconnaissance', re.I), "NT-CORE", "discover"),
    (re.compile(r'analyz|understand|classif|detect|cluster|topic_model|ner\b|segment', re.I), "NT-CORE", "detect"),
    (re.compile(r'metric|measure|score|benchmark|eval|quantif|statistic', re.I), "NT-CORE", "measure"),
    (re.compile(r'predict|forecast|forecast|trend|market', re.I), "NT-CORE", "predict"),
    (re.compile(r'plan|roadmap|scheduler|task_plan|goal', re.I), "NT-CORE", "plan"),
    # 推理
    (re.compile(r'reason|logic|infer|deduc|inference|chain_of_thought|debate|critique', re.I), "NT-CORE", "critique"),
    (re.compile(r'explain|interpret|insight|attribution|xai\b', re.I), "NT-CORE", "explain"),
    # 通用知识/百科 (Cycle 161s): wiki镜像/宗教哲学/百科条目 = 知识解释
    (re.compile(r'wikipedia|wikip|encyclopedia|wiki\b', re.I), "NT-CORE", "explain"),
    (re.compile(r'karma|buddha|shinto|jain|hindu|veda|sanskrit|islam|quran|religion|philosoph|ethics|theolog|sutta|dharma|zen|tao|confuci', re.I), "NT-CORE", "explain"),
    (re.compile(r'google books|open library|archive\.org|gutenberg|libgen|ebook|textbook', re.I), "NT-MEMORY", "recall"),
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
    (re.compile(r'security|vuln|audit|scan|pen_test|pentest|exploit|firewall|shield|protect|secur|pwn|hack|breach|malware|ransom', re.I), "NT-SHIELD", "audit"),
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
    # Cycle 208 批 (2026-08-06 吸收 97 源) — 专家判定 (防 keyword 误伤)
    "mypaios/mypaios": ("NT-CORE", "plan"),         # AI-native OS/knowledge workspace
    "lizhipay/acg-faka": ("NT-ACT", "execute"),     # 发卡系统 (web panel)
    "yorukot/superfile": ("NT-IO", "invoke"),       # TUI 文件管理器
    "Integuru-AI/Integuru": ("NT-ACT", "delegate"), # 集成 agent 编排
    "HKUDS/DeepCode": ("NT-MIND", "transform"),     # LLM 代码修复 (DeepCode)
    "ciembor/agent-rules-books": ("NT-CORE", "plan"), # AGENTS.md 规则书集合
    "freestylefly/awesome-gpt-image-2": ("NT-MIND", "generate"), # GPT 图像生成合集
    "semantica-agi/semantica": ("NT-MEMORY", "recall"), # graph-native 上下文基础设施
    "ln-dev7/circle": ("NT-IO", "invoke"),          # web UI 组件/应用
    "KnowledgeXLab/MemHarness": ("NT-MEMORY", "recall"), # 记忆重建 benchmark
    "666ghj/MiroFish": ("NT-WORLD", "observe"),     # 鱼眼/多模态感知
    "esengine/DeepSeek-Reasonix": ("NT-CORE", "critique"), # DeepSeek 推理工程
    "eugeneyan/news-agents": ("NT-WORLD", "observe"), # 新闻 agent (MCP)
    "microsoft/graphrag": ("NT-MEMORY", "recall"),  # graph RAG
    "langchain-ai/deepagents": ("NT-IO", "delegate"), # agent harness
    "overspace-labs/HaESkill": ("NT-SHIELD", "audit"), # HaE (Burp HTTP 编辑器) skill
    "opendatalab/MinerU": ("NT-MIND", "transform"), # PDF/文档解析
    "TencentCloud/Octop": ("NT-ACT", "delegate"),   # 自托管多用户多 agent 助手
    "goauthentik/authentik": ("NT-SHIELD", "constrain"), # 身份认证/SSO
    "firecrawl/anydoc": ("NT-WORLD", "retrieve"),   # 文档转换
    "webpack/webpack": ("NT-MIND", "transform"),    # 打包器
    "AeternaLabsHQ/pullmd": ("NT-MIND", "transform"), # URL/文件 → markdown
    "TencentCloud/TencentDB-Agent-Memory": ("NT-MEMORY", "recall"), # agent 记忆中枢
    "agentchatme/agentchat-hermes": ("NT-IO", "delegate"), # hermes agent 编排
    "asimons81/hermes-dreaming": ("NT-MIND", "integrate"), # 记忆梦想合成
    "kaishi00/hermes-community-plugins": ("NT-IO", "delegate"), # 社区插件
    "Hmbown/Wizards-of-the-Ghosts": ("NT-MIND", "generate"), # agent 创作
    "markoblogo/abvx-agent-skills": ("NT-IO", "delegate"), # agent skills 集
    "Romanescu11/hermes-skill-factory": ("NT-MIND", "integrate"), # 技能工厂
    "42-evey/hermes-plugins": ("NT-IO", "delegate"), # hermes 插件
    "witt3rd/oh-my-hermes": ("NT-IO", "delegate"),  # hermes 配置/编排
    "tlehman/litprog-skill": ("NT-MIND", "integrate"), # 文学编程 skill
    "DeployFaith/hermes-bible-skill": ("NT-MIND", "integrate"), # 圣经 skill
    "x1xhlol/system-prompts-and-models-of-ai-tools": ("NT-CORE", "critique"), # 系统提示词集
    "leonickson1/Swiftlet": ("NT-IO", "invoke"),    # Swift/macOS app
    "bozhouDev/codex-orange-book": ("NT-ACT", "execute"), # Codex 使用指南
    "sxyazi/yazi": ("NT-IO", "invoke"),             # TUI 文件管理器
    "rasbt/MachineLearning-QandAI-book": ("NT-CORE", "explain"), # ML Q&A 书
    "iOfficeAI/OfficeCLI": ("NT-ACT", "execute"),   # Office 办公 CLI
    "AUTOMATIC1111/stable-diffusion-webui": ("NT-MIND", "generate"), # SD webui
    "zhulin025/Codex-QQ-Skin": ("NT-IO", "invoke"), # Codex 皮肤
    "arxhr007/Aliens_eye": ("NT-WORLD", "observe"), # 840+ 社媒账号搜索 (OSINT)
    "iOfficeAI/AionUi": ("NT-IO", "invoke"),        # 24/7 Cowork UI
    "SteinsHead/ghostty-studio": ("NT-IO", "invoke"), # ghostty 终端主题/配置
    "ifixai-ai/iFixAi": ("NT-SHIELD", "verify"),    # AI agent 独立审计
    "public-apis/public-apis": ("NT-WORLD", "search"), # 公共 API 目录
    "codecrafters-io/build-your-own-x": ("NT-CORE", "plan"), # 造轮子教程
    "kamranahmedse/developer-roadmap": ("NT-CORE", "plan"), # 开发路线图
    "EbookFoundation/free-programming-books": ("NT-MEMORY", "recall"), # 免费书
    "donnemartin/system-design-primer": ("NT-CORE", "explain"), # 系统设计
    "jwasham/coding-interview-university": ("NT-CORE", "explain"), # 面试学习
    "jlevy/the-art-of-command-line": ("NT-ACT", "execute"), # 命令行艺术
    "practical-tutorials/project-based-learning": ("NT-CORE", "plan"), # 项目制学习
    "getify/You-Dont-Know-JS": ("NT-MEMORY", "recall"), # JS 深度书
    "trimstray/the-book-of-secret-knowledge": ("NT-CORE", "explain"), # 秘密知识手册
    "yangshun/tech-interview-handbook": ("NT-CORE", "explain"), # 面试手册
    "awesome-selfhosted/awesome-selfhosted": ("NT-ACT", "delegate"), # 自托管清单
    "trekhleb/javascript-algorithms": ("NT-CORE", "measure"), # JS 算法
    "30-seconds/30-seconds-of-code": ("NT-CORE", "measure"), # 代码片段
    "github/gitignore": ("NT-ACT", "execute"),      # gitignore 模板
    "ollama/ollama": ("NT-IO", "inquire"),          # 本地 LLM 运行时
    "langchain-ai/langchain": ("NT-MIND", "integrate"), # agent 框架
    "n8n-io/n8n": ("NT-ACT", "delegate"),           # 工作流自动化
    "openclaw/openclaw": ("NT-IO", "delegate"),     # agent 网关
    "langgenius/dify": ("NT-MIND", "integrate"),    # LLM app 平台
    "langflow-ai/langflow": ("NT-MIND", "integrate"), # 可视化 agent 编排
    "mem0ai/mem0": ("NT-MEMORY", "recall"),         # agent 记忆层
    "browser-use/browser-use": ("NT-WORLD", "retrieve"), # 浏览器 agent
    "crewAIInc/crewAI": ("NT-IO", "delegate"),      # 多 agent 编排
    "geekan/MetaGPT": ("NT-IO", "delegate"),        # 多 agent 软件公司
    "microsoft/autogen": ("NT-IO", "delegate"),     # 多 agent 框架
    "Aider-AI/aider": ("NT-ACT", "execute"),        # AI pair programming
    "microsoft/markitdown": ("NT-MIND", "transform"), # 文件→markdown
    "open-webui/open-webui": ("NT-IO", "invoke"),   # web UI
    "soxoj/maigret": ("NT-WORLD", "observe"),       # 用户信息 OSINT
    "TauricResearch/TradingAgents": ("NT-CORE", "predict"), # 交易 agent
    "browserbase/stagehand": ("NT-WORLD", "retrieve"), # 浏览器自动化
    "firecrawl/firecrawl": ("NT-WORLD", "retrieve"), # 爬取/搜索 API
    "huggingface/transformers": ("NT-MIND", "generate"), # 模型库
    "vllm-project/vllm": ("NT-MIND", "generate"),   # LLM 推理引擎
    "ggerganov/llama.cpp": ("NT-MIND", "generate"), # LLM 推理
    "run-llama/llama_index": ("NT-MEMORY", "recall"), # RAG 框架
    "karpathy/nanoGPT": ("NT-MIND", "generate"),    # GPT 训练
    "infiniflow/ragflow": ("NT-MEMORY", "recall"),  # RAG 引擎
    "supermemoryai/supermemory": ("NT-MEMORY", "recall"), # 记忆层
    "ComposioHQ/awesome-claude-skills": ("NT-CORE", "plan"), # Claude skills 集
    "perplexityai/bumblebee": ("NT-WORLD", "search"), # 搜索
    "comfyanonymous/ComfyUI": ("NT-MIND", "generate"), # 节点式图像生成
    "lobehub/lobe-chat": ("NT-IO", "invoke"),       # AI 聊天 UI
    # NT-CORE 确定性映射 (Cycle 161q)
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
    # NT-SHIELD: 安全工具确定性映射 (Cycle 161o)
    "nmap": ("NT-SHIELD", "audit"),
    "sqlmap": ("NT-SHIELD", "audit"),
    "zaproxy": ("NT-SHIELD", "audit"),
    "grype": ("NT-SHIELD", "audit"),
    "trivy": ("NT-SHIELD", "audit"),
    "lynis": ("NT-SHIELD", "audit"),
    "wpscan": ("NT-SHIELD", "audit"),
    "impacket": ("NT-SHIELD", "audit"),
    "BloodHound": ("NT-SHIELD", "audit"),
    "Empire": ("NT-SHIELD", "audit"),
    "Amass": ("NT-SHIELD", "audit"),
    "phasar": ("NT-SHIELD", "audit"),
    "scorecard": ("NT-SHIELD", "audit"),
    "cosign": ("NT-SHIELD", "verify"),
    "sigstore": ("NT-SHIELD", "verify"),
    "opa": ("NT-SHIELD", "constrain"),
    "secureCodeBox": ("NT-SHIELD", "verify"),
    # Cycle 206 批 (2026-08-04 吸收 47 源) — 专家判定 (sub-agent 四字段表)
    "kostja94/marketing-skills": ("NT-IO", "invoke"),
    "CyberStrike": ("NT-SHIELD", "constrain"),
    "zhaoxuya520/reverse-skill": ("NT-SHIELD", "audit"),
    "uditgoenka/autoresearch": ("NT-MIND", "integrate"),
    "averygan/reclip": ("NT-WORLD", "retrieve"),
    "github/spec-kit": ("NT-CORE", "plan"),
    "Kritt-ai/open-kritt": ("NT-SHIELD", "audit"),
    "yc-software/qm": ("NT-ACT", "execute"),
    "alibaba/open-code-review": ("NT-CORE", "critique"),
    "affaan-m/ECC": ("NT-MIND", "integrate"),
    "AgentSwarms-fyi/agentswarms": ("NT-ACT", "delegate"),
    "trycompai/crm": ("NT-ACT", "execute"),
    "xai-org/grok-build": ("NT-ACT", "execute"),
    "jakubkrehel/skills": ("NT-IO", "invoke"),
    "jakubkrehel/oklch-skill": ("NT-IO", "invoke"),
    "jakubkrehel/make-interfaces-feel-better": ("NT-IO", "invoke"),
    "robert-mcdermott/ai-knowledge-graph": ("NT-MEMORY", "search"),
    "vxcontrol/pentagi": ("NT-SHIELD", "audit"),
    "toeverything/AFFiNE": ("NT-MEMORY", "persist"),
    "huangruiteng/loopx": ("NT-ACT", "execute"),
    "google/magika": ("NT-SHIELD", "verify"),
    "phibrowser": ("NT-WORLD", "observe"),
    "lightpanda-io/browser": ("NT-WORLD", "retrieve"),
    "firecrawl/pdf-inspector": ("NT-MEMORY", "recall"),
    "aigclink/geolook": ("NT-MIND", "integrate"),
    "diegosouzapw/OmniRoute": ("NT-IO", "inquire"),
    "stablyai/orca": ("NT-ACT", "delegate"),
    "citrolabs/ego-lite": ("NT-WORLD", "retrieve"),
    "CoreBunch/Instatic": ("NT-SHIELD", "constrain"),
    "Lordog/dive-into-llms": ("NT-MEMORY", "recall"),
    "anthropics/claude-cookbooks": ("NT-CORE", "plan"),
    "NanoNets/Graft": ("NT-MEMORY", "search"),
    "ever-co/ever-gauzy": ("NT-ACT", "execute"),
    "superdesigndev/superdesign": ("NT-IO", "invoke"),
    "skalesapp/skales": ("NT-ACT", "execute"),
    "claraverse-space/ClaraVerse": ("NT-MEMORY", "recall"),
    "FareedKhan-dev/kimi-k3-in-c": ("NT-MIND", "generate"),
    "LasCC/HackTools": ("NT-SHIELD", "audit"),
    "taranis-ai/taranis-ai": ("NT-WORLD", "observe"),
    "ruvnet/ruflo": ("NT-SHIELD", "constrain"),
    "projectdiscovery/nuclei": ("NT-SHIELD", "audit"),
    "whiteguo233/OpenBiliClaw": ("NT-WORLD", "observe"),
    # NT-SHIELD constrain: 治理/对齐/约束 (Cycle 161p)
    "constitutional-ai": ("NT-SHIELD", "constrain"),
    "llm-guard": ("NT-SHIELD", "constrain"),
    "guardrails": ("NT-SHIELD", "constrain"),
    "guidance": ("NT-SHIELD", "constrain"),
    "keycloak": ("NT-SHIELD", "constrain"),
    "casbin": ("NT-SHIELD", "constrain"),
    "certbot": ("NT-SHIELD", "verify"),
    "letsencrypt": ("NT-SHIELD", "constrain"),
    "vault": ("NT-SHIELD", "constrain"),
    "sops": ("NT-SHIELD", "constrain"),
    "snyk": ("NT-SHIELD", "audit"),
    "DependencyCheck": ("NT-SHIELD", "audit"),
    "cdxgen": ("NT-SHIELD", "audit"),
    "spdx-sbom-generator": ("NT-SHIELD", "audit"),
    "purl-spec": ("NT-SHIELD", "constrain"),
    "in-toto": ("NT-SHIELD", "verify"),
    "dependabot": ("NT-SHIELD", "audit"),
    "flare-vm": ("NT-SHIELD", "audit"),
    "sigma": ("NT-SHIELD", "audit"),
    "detection-rules": ("NT-SHIELD", "audit"),
    "attack_range": ("NT-SHIELD", "audit"),
    "evals": ("NT-SHIELD", "verify"),
    # NT-IO/NT-MIND 确定性映射 (Cycle 161q)
    "kafka": ("NT-IO", "synchronize"),
    "nats-server": ("NT-IO", "synchronize"),
    "rabbitmq-server": ("NT-IO", "synchronize"),
    "mosquitto": ("NT-IO", "synchronize"),
    "emqx": ("NT-IO", "synchronize"),
    "nsq": ("NT-IO", "synchronize"),
    "libzmq": ("NT-IO", "synchronize"),
    "redis": ("NT-IO", "invoke"),
    "temporal": ("NT-IO", "synchronize"),
    "prefect": ("NT-IO", "synchronize"),
    "dagster": ("NT-IO", "synchronize"),
    "airflow": ("NT-IO", "synchronize"),
    "tree-of-thoughts": ("NT-CORE", "critique"),
    "human-eval": ("NT-CORE", "measure"),
    "peft": ("NT-MIND", "transform"),
    "LoRA": ("NT-MIND", "transform"),
    "OpenInstruct": ("NT-MIND", "generate"),
    "FastChat": ("NT-MIND", "generate"),
    # 优化/调参类 (Cycle 161t)
    "optuna": ("NT-MIND", "transform"),
    "BayesianOptimization": ("NT-MIND", "transform"),
    "keras-tuner": ("NT-MIND", "transform"),
    "talos": ("NT-MIND", "transform"),
    "AutoML": ("NT-MIND", "integrate"),
    # 元学习/自进化 (Cycle 161t)
    "meta-dataset": ("NT-MIND", "integrate"),
    "MAML-Pytorch": ("NT-MIND", "transform"),
    "learn2learn": ("NT-MIND", "transform"),
    "pytorch-meta": ("NT-MIND", "transform"),
    "awesome-meta-learning": ("NT-MIND", "integrate"),
    # 神经模拟 (Cycle 161t)
    "brian2": ("NT-MEMORY", "simulate"),
    "BrainPy": ("NT-MEMORY", "simulate"),
    "PyNN": ("NT-MEMORY", "simulate"),
    "BluePy": ("NT-MEMORY", "simulate"),
    "OpenWorm": ("NT-MEMORY", "simulate"),
    "neuropixels": ("NT-CORE", "measure"),
    # 进化计算/自适应 (Cycle 161t)
    "deap": ("NT-MIND", "integrate"),
    "jenetics": ("NT-MIND", "integrate"),
    "pagmo2": ("NT-MIND", "integrate"),
    "Platypus": ("NT-MIND", "integrate"),
    "cmaes": ("NT-MIND", "transform"),
    # 认知/神经 (Cycle 161t)
    "opencog": ("NT-MEMORY", "simulate"),
    "bids-validator": ("NT-CORE", "verify"),
    # Cycle 232 批 (2026-08-06 吸收 436 源) — 专家判定 (防 keyword 误伤)
    "expo/expo": ("NT-IO", "invoke"),                 # React Native 跨端框架
    "milvus-io/milvus": ("NT-MEMORY", "recall"),      # 向量数据库 (KB 检索)
    "karpathy/autoresearch": ("NT-MIND", "integrate"), # 自主实验循环
    "hacksider/Deep-Live-Cam": ("NT-MIND", "generate"), # 实时人脸替换 (生成)
    "openinterpreter/openinterpreter": ("NT-ACT", "execute"), # 自然语言代码执行
    "livekit/agents": ("NT-IO", "delegate"),          # 语音/实时 agent 框架
    "microsoft/Resource2Skill": ("NT-MIND", "integrate"), # 文档→skill 转换
    "google-research/timesfm": ("NT-CORE", "predict"), # 时序预测基础模型
    "coqui-ai/TTS": ("NT-MIND", "generate"),          # TTS 语音合成
    "openai/swarm": ("NT-ACT", "delegate"),           # 多 agent 编排
    "anthropics/claude-code": ("NT-ACT", "execute"),  # Claude Code CLI agent
    "unslothai/unsloth": ("NT-MIND", "transform"),    # LLM 微调加速
    "microsoft/agent-framework": ("NT-IO", "delegate"), # agent 运行时框架
    "harbor-framework/harbor": ("NT-CORE", "plan"),   # agent 系统架构框架
    "CodebuffAI/codebuff": ("NT-ACT", "execute"),     # 自主编码 agent
    "facebookresearch/map-anything": ("NT-WORLD", "observe"), # 分割/感知
    "anthropics/prompt-eng-interactive-tutorial": ("NT-CORE", "explain"), # 提示工程教程
    "ItusiAI/MokerSaaS": ("NT-ACT", "execute"),       # SaaS 出海启动模板 (rename from get-saas)
    "Anil-matcha/ai-creator-academy": ("NT-CORE", "explain"), # AI 创作免费课程 (rename from awesome-hermes-agent)
}


def normalize_repo_title(title):
    """'Karpathy AutoResearch' 或 'GitHub - owner/repo: desc' → owner/repo"""
    t = re.sub(r'^GitHub\s*-\s*', '', title)
    t = t.split(':')[0].strip()
    return t


def map_node(node_type, title, content, url):
    """返回 (branch, capability, evidence) 或 None"""
    owner_repo = None
    if url and 'github.com' in url:
        url_low = url.lower()
        last = url_low.rstrip('/').rsplit('/', 1)[-1] if url_low.rstrip('/') else ''
        # Pass 1: 完整 owner/repo key 用 URL 判真 (ground truth, 任意 node_type)
        for k, (br, cap) in KNOWN_REPOS.items():
            kl = k.lower()
            if '/' in k and kl in url_low:
                return br, cap, f'known_repo:{k}'
        # Pass 2: 裸 key 须等于 URL 末段 (精确判别, 防 "skills" 误吞 "marketing-skills")
        for k, (br, cap) in KNOWN_REPOS.items():
            kl = k.lower()
            if '/' not in k and kl == last:
                return br, cap, f'known_repo:{k}'
    if node_type == 'repository':
        owner_repo = normalize_repo_title(title)
        low = owner_repo.lower()
        for k, (br, cap) in KNOWN_REPOS.items():
            kl = k.lower()
            if kl in low or low.endswith(kl) or low == kl.split('/')[-1]:
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
    # 本源感知兜底: 无关键词命中时按 node_type + 本源给语义合理的能力 (Cycle 161n)
    if node_type == 'repository':
        return 'NT-WORLD', 'retrieve', 'fallback:repo'
    if node_type == 'paper':
        return 'NT-CORE', 'critique', 'fallback:paper'
    # node_type 语义默认 (占位节点无关键词信号, node_type 是唯一判别)
    type_default = {
        'concept': ('NT-MEMORY', 'recall'),
        'web': ('NT-WORLD', 'search'),
        'external': ('NT-WORLD', 'retrieve'),
        'skill': ('NT-ACT', 'execute'),
        'doi': ('NT-CORE', 'critique'),
        'arxiv': ('NT-CORE', 'critique'),
        'wikipedia': ('NT-MEMORY', 'recall'),
        'reference': ('NT-MEMORY', 'recall'),
        'book': ('NT-MEMORY', 'recall'),
        'guide': ('NT-IO', 'invoke'),
        'method': ('NT-MIND', 'integrate'),
        'framework': ('NT-MIND', 'integrate'),
        'insight': ('NT-CORE', 'explain'),
        'thinking_trace': ('NT-MIND', 'integrate'),
        'theory': ('NT-CORE', 'critique'),
        'person': ('NT-CORE', 'explain'),
        'organization': ('NT-CORE', 'explain'),
        'evolution_pattern': ('NT-MIND', 'integrate'),
        'conversation_evolution': ('NT-MIND', 'integrate'),
        'resource': ('NT-MEMORY', 'recall'),
        'source': ('NT-MEMORY', 'recall'),
        'image': ('NT-IO', 'invoke'),
        'wiki_page': ('NT-CORE', 'explain'),
        'algorithm': ('NT-CORE', 'measure'),
        'note': ('NT-MEMORY', 'recall'),
        'event_record': ('NT-CORE', 'measure'),
        'detection_finding': ('NT-CORE', 'detect'),
        'goal_result': ('NT-CORE', 'measure'),
        'github': ('NT-ACT', 'execute'),
        'arxiv': ('NT-CORE', 'critique'),
    }
    if node_type in type_default:
        return *type_default[node_type], f'fallback:type:{node_type}'
    core, _, _ = map_source_core(title, content[:1200], '', node_type)
    if core is None:
        core, _, _ = fallback_source(title, node_type)
    src_cap = {
        'E8': ('NT-CORE', 'measure'),
        'VSA': ('NT-MEMORY', 'recall'),
        'GWT': ('NT-CORE', 'critique'),
        'ConsciousnessTree': ('NT-MIND', 'integrate'),
        'Reality': ('NT-MEMORY', 'recall'),
    }
    return *src_cap.get(core, ('NT-CORE', 'discover')), f'fallback:core:{core}'


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--apply', action='store_true', help='写入 KB (absorbed_capabilities 字段)')
    ap.add_argument('--report', action='store_true', help='只输出覆盖率报告')
    args = ap.parse_args()

    conn = sqlite3.connect(KB_PATH)
    rows = conn.execute("""SELECT id, node_type, title, content, url, metadata FROM nodes
                           WHERE id LIKE 'batch_%'""").fetchall()
    print(f'[mapping] {len(rows)} batch nodes', flush=True)

    mapped = {}
    per_branch = {}
    per_cap = {}
    per_source = {}
    unmapped = []
    for nid, node_type, title, content, url, meta_json in rows:
        # GitHub topics/description 补充本源判定 (repository content 是占位模板)
        topics = []
        if meta_json:
            md = json.loads(meta_json)
            topics = md.get('topics') or []
            if md.get('description'):
                topics.append(md['description'])
        topic_blob = ' '.join(topics)
        res = map_node(node_type, title, content or '', url)
        if res is None:
            unmapped.append((nid, title))
            continue
        branch, cap, ev = res
        # 本源溯源层 (Cycle 161i): 5 道之本源 + 演化路径
        core, core_domain, trace_kws = map_source_core(title, content or '', url, node_type)
        if core is None and topic_blob:
            core, core_domain, trace_kws = map_source_core(topic_blob, '', '', node_type)
        if core is None:
            # 兜底: 标题线索词分源 (本源哲学: 每节点终有所属本源)
            core, core_domain, trace_kws = fallback_source(title, node_type)
        mapped[nid] = {'branch': branch, 'capability': cap, 'evidence': ev,
                       'node_type': node_type, 'title': title[:60], 'url': url,
                       'source_core': core, 'source_domain': core_domain,
                       'trace_keywords': trace_kws}
        per_branch.setdefault(branch, []).append(cap)
        per_cap[cap] = per_cap.get(cap, 0) + 1
        if core:
            per_source[core] = per_source.get(core, 0) + 1

    if args.apply:
        # R-P97: 写回委托 Rust CLI (update-node-metadata) — 单一事实源。
        # Python 仅算映射结果 (286 专家键 + 规则 + 本源溯源), 写回交 Rust merge。
        now = int(time.time())
        updates = []
        for nid, m in mapped.items():
            patch = {
                'absorbed_capability': {
                    'branch': m['branch'],
                    'capability': m['capability'],
                    'evidence': m['evidence'],
                    'mapped_at': now,
                }
            }
            if m.get('source_core'):
                patch['knowledge_source'] = {
                    'source_core': m['source_core'],
                    'primary_domain': m.get('source_domain'),
                    'trace_path': m.get('trace_keywords', []),
                    'mapped_at': now,
                }
            updates.append({'node_id': nid, 'patch': patch})
        ins, missing = rust_update_node_metadata(updates)
        print(f'[mapping] wrote {ins} capability mappings to KB (missing={missing})', flush=True)

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
    print(f'\n--- 5 本源溯源分布 (道之本源脉络) ---')
    for core, cnt in sorted(per_source.items(), key=lambda x: -x[1]):
        print(f'  {core:<18} {cnt:>4}')
    unknown = len(rows) - sum(per_source.values())
    print(f'  {"(unknown)":<18} {unknown:>4}')
    print(f'\n--- 未映射节点 ---')
    for nid, t in unmapped[:15]:
        print(f'  {nid}  {t[:60]}')

    conn.close()


if __name__ == '__main__':
    main()
