#!/usr/bin/env python3
"""ppt_theme.py — PPT 风格设计大师引擎（可复用）

核心能力：
1. TypeScale   — 专业字体层级（标题/小节/卡标题/正文/辅助/微标注 + 行距/字距）
2. Theme       — 由项目背景推导的完整风格（配色 + 视觉语言 + 圆角 + 字距）
3. derive_theme(project, background) — 规则引擎：从背景文本提取领域信号，匹配最贴合的主题
4. t()         — 按样式类生成带格式的文本元组，配合 put_text 使用

用法：
    theme = derive_theme("NeoTrix", "AI-native 自进化 Agent 研发闭环 ...")
    lines = [t(theme, "h1", "AgentTeams 编排层"), t(theme, "body", "Manager 拆解·委派·追踪")]
    box(slide, x, y, w, h, lines, theme.surface, theme.ink)

主题推导信号（关键词打分）：
- 意识/认知/AI/Agent/自进化/注意力 → Consciousness（意识流：navy+amber+cyan）
- E8/六爻/符号/确定性推理/VSA       → Hexagram（卦象：navy+gold）
- 研发/工程/软件/CI/测试/代码        → Engineering（深空工程：blue-gray）
- 开源/MIT/社区/复用                 → Open（加分信号：green）
- 金融/商务/企业/专业服务            → Corporate（商务：deepblue+gold）
"""
from dataclasses import dataclass, field
from typing import Optional


# ---------------- 字体层级 ----------------
@dataclass(frozen=True)
class TypeScale:
    title: float = 29.0      # 页标题（模板自带）
    kicker: float = 12.0     # 小节标签/胶囊
    lead: float = 11.5       # 顶部一句导语
    h1: float = 12.5         # 卡片标题条
    h2: float = 11.5         # 层级/节点标签
    body: float = 10.5       # 正文条目
    support: float = 10.0    # 底部辅助条
    micro: float = 9.0       # 微标注/标签
    line_body: float = 1.18  # 正文行距
    line_dense: float = 1.02  # 紧凑行距
    tracking: float = 0.10   # 小节标签字距（em）


# ---------------- 主题 ----------------
@dataclass
class Theme:
    name: str
    base: str                # 深底色（顶部导语条）
    ink: str                 # 主文字
    primary: str             # 主色
    accent: str              # 强调色（注意力/高光）
    support1: str            # 二级色
    support2: str            # 三级色
    positive: str            # 正向（价值/通过）
    negative: str            # 负向（痛点/风险）
    surface: str             # 卡片底
    surface2: str            # 辅助条底
    muted: str               # 次要文字
    radii: float = 0.12      # 圆角
    signals: list = field(default_factory=list)  # 命中关键词
    keywords: list = field(default_factory=list)  # 该主题关键词


THEMES = {
    "consciousness": Theme(
        name="Consciousness · 意识流",
        base="1B1F3B", ink="0F172A", primary="2E5BFF",
        accent="F5B301", support1="0EA5E9", support2="06B6D4",
        positive="10B981", negative="EF4444",
        surface="F1F5F9", surface2="E2E8F0", muted="64748B",
        keywords=["意识", "认知", "注意力", "attention", "自进化", "self-evol",
                  "agent", "AI", "智能", "推理", "reason", "hypercube", "元认知",
                  "GWT", "consci", "脑", "mind"],
    ),
    "hexagram": Theme(
        name="E8 Hexagram · 卦象",
        base="0F1B2D", ink="12203A", primary="2456C9",
        accent="D4A72C", support1="3B82F6", support2="0EA5E9",
        positive="10B981", negative="DC2626",
        surface="EEF2F7", surface2="E0E7EF", muted="5B6B82",
        keywords=["E8", "六爻", "卦", "hexagram", "符号", "symbolic",
                  "确定性", "determin", "VSA", "向量符号"],
    ),
    "engineering": Theme(
        name="Deep Engineering · 深空工程",
        base="111827", ink="111827", primary="2563EB",
        accent="0EA5E9", support1="14B8A6", support2="6366F1",
        positive="059669", negative="DC2626",
        surface="F3F4F6", surface2="E5E7EB", muted="6B7280",
        keywords=["研发", "工程", "软件", "开发", "CI", "测试", "构建",
                  "build", "test", "deploy", "code", "repo", "issue",
                  "infra", "pipeline", "cicd"],
    ),
    "corporate": Theme(
        name="Corporate · 商务",
        base="0B1F3A", ink="17233B", primary="1D4ED8",
        accent="C9A227", support1="0E7490", support2="3B82F6",
        positive="15803D", negative="B91C1C",
        surface="F5F6F8", surface2="E4E7EC", muted="5B6472",
        keywords=["企业", "商务", "金融", "专业", "服务", "客户", "商业", "business", "enterprise"],
    ),
    "minimal": Theme(
        name="Minimal · 极简",
        base="18181B", ink="27272A", primary="3F3F46",
        accent="18181B", support1="52525B", support2="71717A",
        positive="15803D", negative="B91C1C",
        surface="FAFAFA", surface2="F4F4F5", muted="71717A",
        keywords=["极简", "简洁", "clean", "minimal", "mono", "黑白"],
    ),
}


# ---------------- 主题推导 ----------------
def _score(theme: Theme, text: str) -> int:
    low = text.lower()
    return sum(1 for kw in theme.keywords if kw.lower() in low)


def derive_theme(project: str, background: str, prefer: Optional[str] = None) -> Theme:
    """根据项目名 + 背景描述推导主题。prefer 可显式指定（consciousness/hexagram/...）。"""
    if prefer and prefer in THEMES:
        th = THEMES[prefer]
    else:
        text = f"{project} {background}"
        scores = [(t, _score(t, text)) for t in THEMES.values()]
        best = max(scores, key=lambda x: x[1])[0]
        th = best
        if scores and best.signals:
            pass
    # 记录命中信号（用于日志）
    text = f"{project} {background}"
    th.signals = [kw for kw in th.keywords if kw.lower() in text.lower()]
    return th


# ---------------- 样式类 → 文本元组 ----------------
def t(theme: Theme, cls: str, text: str, *, color: Optional[str] = None,
      bold: Optional[bool] = None, size: Optional[float] = None):
    """按样式类生成 (text, size, bold, color, line_height) 元组。覆盖参数优先。"""
    ts = TypeScale()
    spec = {
        "kicker": (ts.kicker, True, theme.primary),
        "lead":   (ts.lead, True, theme.base),
        "h1":     (ts.h1, True, "FFFFFF"),
        "h2":     (ts.h2, True, "FFFFFF"),
        "body":   (ts.body, False, theme.ink),
        "body-bold": (ts.body, True, theme.ink),
        "support": (ts.support, False, theme.ink),
        "micro":  (ts.micro, False, theme.muted),
        "micro-bold": (ts.micro, True, theme.muted),
        "accent": (ts.body, True, theme.accent),
        "positive": (ts.body, True, theme.positive),
        "negative": (ts.body, True, theme.negative),
        "muted":  (ts.body, False, theme.muted),
        "white-body": (ts.body, False, "FFFFFF"),
    }[cls]
    sz, bd, cl = spec
    if color is not None:
        cl = color
    if bold is not None:
        bd = bold
    if size is not None:
        sz = size
    lh = ts.line_body if cls in ("body", "body-bold", "muted", "positive",
                                 "negative", "accent", "white-body", "support") else ts.line_dense
    return (text, sz, bd, cl, lh)


def line_height(cls: str) -> float:
    ts = TypeScale()
    if cls in ("body", "body-bold", "muted", "positive", "negative", "accent", "white-body"):
        return ts.line_body
    return ts.line_dense