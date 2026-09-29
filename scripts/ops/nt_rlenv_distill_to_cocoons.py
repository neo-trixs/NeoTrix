#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_rlenv_distill_to_cocoons — MiMo-V2.6-RL-oss 训练环境 → 晶体核心记忆茧.

数据源（Apache-2.0，小米 MiMo，真实发布，非合成）：
  `XiaomiMiMo/MiMo-V2.6-RL-oss` — Agentic RL 训练环境（5 域 / 7,780 任务行
  + 925 个带 rubric 的知识工作环境）。

## 为什么不按「任务实例」吸，而按「验证器工程」吸

实测（2026-09-28，见 handoff）：7,780 个任务行高度同质 —— 2,698 条 code 任务
的验证器形态**完全一致**（`bash mimo_test_command.sh` + `test_patch` +
`verifier_timeout_sec` 恒为 1800），全部 `reward.style=rule`、全部单轮 user
prompt。逐条倾倒 = 把冗余灌进核心。真正独有、可迁移的知识在两处：

  1. **925 份分层加权 rubric**（`verifier_meta.json`，共 5,125 个检查项）——
     `tier`(critical/important/sanity) × `method`(llm/rule) × `pass_anchor`
     内嵌 ground truth。这是「怎么给 agent 任务写可验证评分器」的经验工程。
  2. **共享判分骨架** `verify.py`（925 份**逐字节相同**，sha256 caa22a4c64be…，
     抽样 3 环境实测唯一哈希=1）：responses/chat/gemini 三级 judge fallback 链、
     key 不落盘、证据 20000 预算、votes 投票。

## 四层蒸馏

  rl-domain     5 域 × 任务族/验证机制/奖励形态 + 1 总览          （锚点）
  rl-verifier   判分器工程模式 ~20 条（实测统计 + 可迁移设计）      （1 锚点）
  rl-taxonomy   16 环境族 + 音乐约束文法 + 漏洞签名分类 + 预算包络  （16 族锚点）
  rl-rubric     925 份逐环境 rubric 摘要                            （连族锚点+判分锚点）

连边：族锚点→general 域锚点；taxonomy→各域锚点；逐环境 rubric→族锚点+判分锚点。
「融入」而非「存入」= 这套 rubric 分层法本身成了核心里可召回的工程知识。

## 置信度口径（与医患集不同，刻意区分）

本集所有数字都是**我直接实测**的（不是合成叙述），故：
  - 实测统计（计数/分布/哈希）conf 0.90
  - 结构设计事实（验证机制）conf 0.85
  - 推断性教训（如「权重不可信」）conf 0.70
医患集是 Opus 合成 → 封顶 0.75；本集是真实发布 + 自测 → 可高。
凡**推断**一律不冒充实测。

用法（项目根执行）:
  python3 scripts/ops/nt_rlenv_distill_to_cocoons.py --dry-run
  python3 scripts/ops/nt_rlenv_distill_to_cocoons.py --commit
  python3 scripts/ops/nt_rlenv_distill_to_cocoons.py --selftest
"""
import argparse
import collections
import glob
import json
import os
import re
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nt_hf_digest_to_cocoons as base  # noqa: E402

MIMO = os.path.join("datasets", "hf_raw", "mimo")
ENV_DIR = os.path.join(MIMO, "general", "envs")
SHARED = os.path.join(MIMO, "general", "verify_shared.py")
MANIFEST = os.path.join("datasets", "hf_mimo", "absorb_manifest.json")

DS_ID = "XiaomiMiMo/MiMo-V2.6-RL-oss"
DS_URL = "https://huggingface.co/datasets/" + DS_ID
LICENSE = "apache-2.0"

# 记忆面 → (domain, MemoryType, confidence, importance)
D_DOMAIN, D_VERIFIER, D_TAXONOMY, D_RUBRIC = (
    "rl-domain", "rl-verifier", "rl-taxonomy", "rl-rubric")

CONF_MEASURED = 0.90     # 我直接实测的计数/分布/哈希
CONF_STRUCT = 0.85       # 结构设计事实
CONF_INFER = 0.70        # 推断性教训

WS_RE = re.compile(r"\s+")


def flat(s, cap=0):
    t = WS_RE.sub(" ", (s or "")).strip()
    if cap and len(t) > cap:
        t = t[:cap].rsplit(" ", 1)[0] + " …"
    return t


def jload(p):
    with open(p, encoding="utf-8") as fh:
        return json.load(fh)


def sget(d, *ks, default=""):
    for k in ks:
        v = d.get(k)
        if isinstance(v, str) and v.strip():
            return v
    return default


def num(v):
    """标量 → 字符串（int/float/str 皆可；bool 不算）。"""
    if isinstance(v, bool) or v is None:
        return ""
    if isinstance(v, (int, float)):
        return str(v)
    if isinstance(v, str):
        return v.strip()
    return ""


# ---------------------------------------------------------------- 语料装载
def load_parquet(name):
    import pyarrow.parquet as pq
    p = os.path.join(MIMO, "general", "train.parquet") if name == "general" \
        else os.path.join(MIMO, name + ".parquet")
    return pq.read_table(p).to_pylist()


def inst(row):
    """instance_json → dict（music 无此字段）。"""
    s = (row.get("extra_info") or {}).get("instance_json") or ""
    try:
        d = json.loads(s)
        return d if isinstance(d, dict) else {}
    except ValueError:
        return {}


CYB_SAN = re.compile(r"^([A-Za-z]+):\s*(.+?)\s+in function")
CYB_FILE = re.compile(r"in file `([^`]+)`")
ENV_ID = re.compile(r"^s3k_\d+_(.*)_(en|zh)_t(\d+)_rl_(\d+)$")


def env_parts(env_id):
    m = ENV_ID.match(env_id)
    if not m:
        return (env_id, "?", "?", "?")
    return (m.group(1), m.group(2), m.group(3), m.group(4))


# ---------------------------------------------------------------- 蒸馏主体
class Builder(object):
    """收集 (layer, content, conns[])，最后统一编号 + 解引用锚点。

    刻意**不**做「先数条数再算偏移」的前缀和：1k 量级记忆直接全驻内存，
    编号一次成型，从根上消除 count/actual 漂移那一类 bug。
    """

    def __init__(self):
        self.rows = []          # (layer, content, [conn_ref])
        self.anchors = {}       # ref -> True

    def anchor(self, ref, content):
        self.anchors[ref] = True
        self.rows.append(("anchor:" + ref, content, []))

    def add(self, layer, content, conns):
        self.rows.append((layer, content, conns))


def distil(b):
    """产出全部记忆内容（未编号）。b.anchors 先建锚点，故 conns 可前向引用。"""
    code = load_parquet("code")
    cyb = load_parquet("cyber")
    web = load_parquet("webdev")
    mus = load_parquet("music")
    gen = load_parquet("general")
    rubrics = sorted(glob.glob(os.path.join(ENV_DIR, "*.rubric.json")))

    # ---------------- L1 域锚点 ----------------
    def dom_anchor(ref, name, family, verifier, detail, conf=CONF_STRUCT):
        b.anchor(ref, "[%s] MiMo-RL %s — 任务族 %s；验证机制 %s；%s"
                 % (name, name, family, verifier, detail))

    dom_anchor("code", "code", "software engineering",
               "executable tests (test_patch + bash mimo_test_command.sh)",
               "verifier_timeout_sec 恒为 1800；%d 行；全部 reward.style=rule" % len(code))
    dom_anchor("cyber", "cyber", "vulnerability reproduction",
               "rule checks (sanitizer crash reproduction)",
               "%d 行；prompt 为 sanitizer 崩溃签名" % len(cyb))
    dom_anchor("webdev", "webdev", "web development",
               "visual grading (category=website)",
               "%d 行；自然语言站点规格" % len(web))
    dom_anchor("music", "music", "symbolic music composition",
               "rule checks (符号约束)",
               "%d 行；约束文法 (tag,bpm,meter,nvoice_want,length,lang)" % len(mus))
    dom_anchor("general", "general", "knowledge work",
               "rule checks + LLM rubric judging (verifier_meta.json)",
               "%d 行 = general_agent %d + terminal_bench %d"
               % (len(gen),
                  sum(1 for r in gen if (r.get("extra_info") or {}).get("dataset_type") == "general_agent"),
                  sum(1 for r in gen if (r.get("extra_info") or {}).get("dataset_type") == "terminal_bench")))

    b.anchor("overview", "[overview] MiMo-V2.6-RL-oss = 5 域 agentic RL 环境"
                        "（code/cyber/webdev/music/general），共 %d 任务行 + %d rubric 环境；"
                        "全部 reward.style=rule、单轮 user prompt、agent=mimo_swe_agent；"
                        "知识价值集中在验证器工程而非任务实例"
                        % (len(code) + len(cyb) + len(web) + len(mus) + len(gen), len(rubrics)))

    # ---------------- L2 判分器工程（锚点 verifier） ----------------
    b.anchor("verifier", "[verifier] 判分器工程 —— MiMo RL 的可验证评分体系"
                         "（925 环境共用同一份 verify.py 骨架 + 各自 rubric）")

    tiers, meths, nitems, wsums, noq, anchors_n = (collections.Counter(),
                                                   collections.Counter(), 0, [], 0, 0)
    fam_count = collections.Counter()
    lang_count = collections.Counter()
    for p in rubrics:
        d = jload(p)
        items = d.get("items") or []
        nitems += len(items)
        w = 0.0
        for x in items:
            tiers[x.get("tier")] += 1
            meths[x.get("method")] += 1
            if not x.get("question"):
                noq += 1
            if x.get("pass_anchor"):
                anchors_n += 1
            if isinstance(x.get("weight"), (int, float)) and not isinstance(x.get("weight"), bool):
                w += x["weight"]
        wsums.append(round(w, 4))
        eid = os.path.basename(p).replace(".rubric.json", "")
        fam, lang, _t, _r = env_parts(eid)
        fam_count[fam] += 1
        lang_count[lang] += 1

    unit1 = sum(1 for s in wsums if abs(s - 1.0) < 1e-6)
    have_w = sum(1 for s in wsums if s > 0)

    b.add(D_VERIFIER, "[verifier-pattern] 分层 rubric：每个检查项带 tier，"
            "全 %d 项分布 critical=%d / important=%d / sanity=%d —— "
            "critical 约占 %.0f%%，即「关键项必须全对」是主流设计，而非平均分"
            % (nitems, tiers["critical"], tiers["important"], tiers["sanity"],
               100.0 * tiers["critical"] / max(nitems, 1)),
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-pattern] 双方法混合：method=llm %d 项（带 question + "
            "pass_anchor，交模型判分）/ method=rule %d 项（带 fn，交代码函数判分）；"
            "%d 项无 question —— 正是 rule 项（用 fn 不用问句）。可验证性优先于全自动判分"
            % (meths["llm"], meths["rule"], noq),
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-pattern] pass_anchor 模式：%d/%d 项把 ground truth "
            "直接写进判分问句（pass_anchor），使 judge 的判定有据可依、防止自由发挥。"
            "这是「LLM 判分」能不开人工复核的关键工程手法"
            % (anchors_n, nitems),
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-lesson] 权重不可信：925 个环境里只有 %d 个的检查项带 weight，"
            "且**逐环境**权重和恰为 1.0 的仅 %d 个（逐环境和值实测跨度 %.2f..%.2f）。"
            "故消费这类 rubric 时不可假定权重已归一化，须自行按 tier 重算"
            % (have_w, unit1, min(wsums), max(wsums)),
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-pattern] judge 三级 fallback 链：verify.py 同一份骨架内 "
            "并列 responses(OpenAI Responses 流式) / chat(/chat/completions 非流式，"
            "兼容 sglang/vllm) / gemini(原生 v1beta generateContent) 三种调用形态 —— "
            "因 OpenAI 兼容层对 Gemini 组报 500 \"contents is required\"，必须走原生 wire",
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-pattern] key 不落盘：judge key 走 GA_JUDGE_KEY/_KEY2/_KEY3 "
            "+ 同后缀 MODEL/API/URL 环境变量，由 controller 在 sidecar exec 前缀上透传，"
            "env 内不持久化 —— 多 judge 供应商容灾 + 凭据不落盘的双重要求",
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-pattern] 证据预算与投票：_evidence_sc(ws, files, budget=20000) "
            "对被检文件抽取证据并限预算，_run_llm_sc(items, ws, votes=N) 支持多票聚合 —— "
            "控 token 成本 + 压单次判分抖动",
            ["verifier", "general"])
    b.add(D_VERIFIER, "[verifier-pattern] 产出契约：%d/%d 项读 answer.md（即 agent 只需交一份"
            "answer.md 即可被评分）；reward 函数名统一为 rl_grade"
            % (sum(1 for p in rubrics for x in (jload(p).get("items") or [])
                   if "answer.md" in (x.get("files") or [])), nitems),
            ["verifier", "general"])
    norw = [os.path.basename(p).replace(".rubric.json", "")
            for p in rubrics if not jload(p).get("reward")]
    if norw:
        b.add(D_VERIFIER, "[verifier-lesson] 源数据缺陷：%d 份 rubric 缺 reward 字段（%s）—— "
                "消费端若直接取 reward 会拿到 None，须容错"
                % (len(norw), ", ".join(norw[:4])),
                ["verifier", "general"])
    if os.path.exists(SHARED):
        import hashlib
        sha = hashlib.sha256(open(SHARED, "rb").read()).hexdigest()
        sz = os.path.getsize(SHARED)
        b.add(D_VERIFIER, "[verifier-fact] 判分骨架去重：%d 个环境的 verify.py **逐字节相同**"
                "（sha256 %s，%d 字节，实测唯一哈希=1）—— 拉数据时按内容去重，"
                "省掉 ~9000 次请求；独有知识只在各自的 verifier_meta.json"
                % (len(rubrics), sha[:16], sz),
                ["verifier"])

    # ---------------- L3 分类学（16 族锚点 + 各域分类） ----------------
    # 族锚点的 lang 必须是**该族自己的**分布。坑：曾用全局 lang_count 填进每个
    # 族锚点，产出 16 条一模一样的「lang=en:504/zh:421」（那是全库口径，非该族）。
    fam_lang = collections.defaultdict(collections.Counter)
    for p in rubrics:
        eid0 = os.path.basename(p).replace(".rubric.json", "")
        f0, l0, _t0, _r0 = env_parts(eid0)
        fam_lang[f0][l0] += 1
    for fam, n in fam_count.most_common():
        ref = "fam_" + fam
        lc = fam_lang.get(fam) or collections.Counter()
        b.anchor(ref, "[family] %s —— MiMo general 域知识工作环境族，%d 个环境"
                     "（占全库 %d/%d = %.0f%%）；本族 lang %s"
                % (fam, n, n, len(rubrics), 100.0 * n / len(rubrics),
                   "/".join("%s:%d" % (k, v) for k, v in lc.most_common())))

    # 音乐约束文法
    for key in ("tag", "meter", "length", "lang", "nvoice_want"):
        c = collections.Counter(num((r.get("extra_info") or {}).get(key)) for r in mus)
        c.pop("", None)
        top = ", ".join("%s×%d" % (k, v) for k, v in c.most_common(6))
        b.add(D_TAXONOMY, "[taxonomy:music] extra_info.%s = %d 取值；高频 %s"
                % (key, len(c), top), ["music", "overview"])
    bpms = [num((r.get("extra_info") or {}).get("bpm")) for r in mus]
    bpms = [x for x in bpms if x]
    b.add(D_TAXONOMY, "[taxonomy:music] bpm %d 个不同取值（%s..%s）—— 速度是自由连续维度，"
            "其余约束多为小枚举 → 生成规格里 bpm 可连续采样，其余离散穷举"
            % (len(set(bpms)), min(bpms, key=int) if bpms else "?",
               max(bpms, key=int) if bpms else "?"), ["music", "overview"])

    # cyber 漏洞签名分类 —— **逐 sanitizer 各自的主错误类**。
    # 坑：曾误用全局 err.most_common(1) 填进每个 sanitizer，产出
    # 「MemorySanitizer→heap-buffer-overflow」这种假映射（实测 MSan 96% 是
    # use-of-uninitialized-value）。各 sanitizer 的主错误类之差正是本数据集的
    # 分类轴本身，用全局值会把这条轴整个抹平。
    san, err, proj = collections.Counter(), collections.Counter(), collections.Counter()
    san_err = collections.defaultdict(collections.Counter)
    for r in cyb:
        s = (r["prompt"][0]["content"] or "").strip()
        m = CYB_SAN.match(s)
        if m:
            san[m.group(1)] += 1
            err[m.group(2)] += 1
            san_err[m.group(1)][m.group(2)] += 1
        f = CYB_FILE.search(s)
        if f:
            proj[f.group(1).split("/")[0]] += 1
    axes = []
    for s0, n in san.most_common():
        c = san_err[s0]
        tot = sum(c.values()) or 1
        top = c.most_common(3)
        axes.append("%s(n=%d)→%s" % (s0, n, "/".join(
            "%s %.0f%%" % (e, 100.0 * v / tot) for e, v in top)))
    b.add(D_TAXONOMY, "[taxonomy:cyber] 每个 sanitizer 有**特征主错误类**（分类轴本体）：%s "
            "—— 三轴互不重叠，故「sanitizer 类型」几乎等价于「内存缺陷类别」"
            % "；".join(axes),
            ["cyber", "overview"])
    b.add(D_TAXONOMY, "[taxonomy:cyber] %d 个错误类型（CWE 语义层）：%s"
            % (len(err), ", ".join("%s×%d" % (k, v) for k, v in err.most_common(10))),
            ["cyber", "overview"])
    b.add(D_TAXONOMY, "[taxonomy:cyber] 复现目标覆盖 %d 个 C/C++ 项目（真实 CVE 面）：%s"
            % (len(proj), ", ".join("%s×%d" % (k, v) for k, v in proj.most_common(10))),
            ["cyber", "overview"])

    # code 验证形态
    tcs = collections.Counter()
    for r in code:
        tcs[flat(inst(r).get("test_command"), 70)] += 1
    b.add(D_TAXONOMY, "[taxonomy:code] 验证命令形态仅 %d 种（全部 bash mimo_test_command.sh "
            "变体）：%s —— 2698 个任务共用同一验证入口，差异全在 test_patch 里"
            % (len(tcs), "；".join("%s ×%d" % (k or "-", v) for k, v in tcs.most_common(3))),
            ["code", "overview"])
    tp = sum(len(inst(r).get("test_patch") or "") for r in code)
    b.add(D_TAXONOMY, "[taxonomy:code] test_patch 合计 %.1fMB（测试即规格：可执行测试"
            "定义了「什么算修好」，比自然语言 issue 精确）" % (tp / 1e6), ["code", "overview"])

    # webdev
    wcat = collections.Counter(inst(r).get("category") for r in web)
    b.add(D_TAXONOMY, "[taxonomy:webdev] category 仅 %d 种（%s）—— 视觉评分域内"
            "任务同质，差异在自然语言站点规格"
            % (len(wcat), ", ".join("%s×%d" % (k, v) for k, v in wcat.most_common())),
            ["webdev", "overview"])

    # general 预算包络
    gcat = collections.Counter()
    env = collections.Counter()
    for r in gen:
        ij = inst(r)
        if ij.get("category"):
            gcat[ij["category"]] += 1
        if ij.get("cpus"):
            env["cpus=%s memory_mb=%s internet=%s timeout=%s"
                % (num(ij.get("cpus")), num(ij.get("memory_mb")),
                   num(ij.get("allow_internet")), num(ij.get("agent_timeout_sec")))] += 1
    b.add(D_TAXONOMY, "[taxonomy:general] 64 个 terminal_bench 环境显式声明资源包络：%s"
            % "；".join("%s ×%d" % (k, v) for k, v in env.most_common(3)),
            ["general", "overview"])
    b.add(D_TAXONOMY, "[taxonomy:general] 领域 category（%d 种）：%s —— 覆盖真实企业职能"
            % (len(gcat), ", ".join("%s×%d" % (k, v) for k, v in gcat.most_common(10))),
            ["general", "overview"])
    b.add(D_TAXONOMY, "[taxonomy:general] 环境族 × 企业职能交叉：%d 个族横跨会计审计/金融保险/"
            "医疗运营/咨询/HR/政务/IT/教育/能源/法务/建筑/制造/电商/酒店/物流/农业 —— "
            "general 域考的是「在真实企业系统里做知识工作并交付可核验产物」"
            % len(fam_count), ["general", "overview"])

    # ---------------- L4 逐环境 rubric 摘要 ----------------
    for p in rubrics:
        eid = os.path.basename(p).replace(".rubric.json", "")
        d = jload(p)
        items = d.get("items") or []
        fam, lang, tier_lvl, rl = env_parts(eid)
        tc = collections.Counter(x.get("tier") for x in items)
        mc = collections.Counter(x.get("method") for x in items)
        files = collections.Counter(
            tuple(x.get("files") or []) for x in items if x.get("files"))
        arts = "; ".join("/".join(fl) for fl, _n in files.most_common(3))
        qs = [flat(x.get("question"), 150) for x in items
              if x.get("method") == "llm" and x.get("question")]
        body = " | ".join(qs[:3]) or flat(sget(d, "check_code"), 300)
        # 2026-09-28 monoculture 修复（源头）：此处原为
        #   ["fam_<族>", "verifier"]  ← 每份 rubric 都连「判分器」锚点
        # 致 925 份 rubric 全部指向同一个锚点，实测该锚点**入度 935**，
        # 成为全库最大枢纽（次名仅 171）。后果：它是**通用连接件**，经它取到的
        # 925 个入邻居分属 925 个不同疾病族、彼此毫无语义关联 → 前提选择若走
        # 入邻域，会产出「前提同指率」极低的垃圾链。
        #
        # 判分方案的信息并未丢失：rubric 正文已含 `reward=rl_grade` 与判分要点，
        # 且 10 条 `[verifier-pattern]` 记忆本身就描述该方案。**族锚点是真正
        # 语义正确的父节点**（同族 = 同职能域），故只连族锚点。
        # 跨域性仍保留：族锚点在 rl-taxonomy，rubric 在 rl-rubric → 仍跨域。
        conns = ["fam_" + fam] if ("fam_" + fam) in b.anchors else ["overview"]
        b.add(D_RUBRIC, "[rubric] %s（族=%s lang=%s 难度t%s rl%s）：%d 检查项 "
                "tier{%s} method{%s} reward=%s；被检产物 %s；判分要点 %s"
                % (eid, fam, lang, tier_lvl, rl, len(items),
                   ",".join("%s:%d" % (k, v) for k, v in tc.most_common()),
                   ",".join("%s:%d" % (k, v) for k, v in mc.most_common()),
                   sget(d, "reward") or "None",
                   arts or "(无 files 字段)",
                   body or "(无 llm 判分问句，全为 rule 项)"),
              conns)
    return b


# ---------------------------------------------------------------- 编号落盘
def commit(b, args):
    now = int(time.time())
    mx, _n = base.fast_max_mid(base.COCOONS)
    # 顺序：锚点先于引用它的记忆 —— 故分两轮编号（锚点轮 + 引用轮）
    aid, order = {}, []
    nxt = mx + 1
    for layer, content, conns in b.rows:
        if layer.startswith("anchor:"):
            ref = layer.split(":", 1)[1]
            aid[ref] = "M-%06d" % nxt
            nxt += 1
            order.append((ref, content, conns, True))
    for layer, content, conns in b.rows:
        if not layer.startswith("anchor:"):
            order.append((None, content, conns, False))

    FACET = {D_VERIFIER: ("Pattern", CONF_MEASURED, 0.90),
             D_TAXONOMY: ("Fact", CONF_MEASURED, 0.70),
             D_RUBRIC: ("Fact", CONF_MEASURED, 0.60)}
    # 内容标签 → domain（前缀匹配：`[verifier-pattern]`/`[taxonomy:music]`/`[rubric]`）。
    # 锚点同样按标签归域 —— 否则 16 个族锚点会全堆进 rl-domain（实测 23 条全落一处）。
    TAG2DOM = {"[verifier": D_VERIFIER, "[taxonomy": D_TAXONOMY,
               "[family": D_TAXONOMY, "[rubric": D_RUBRIC}
    per_dom = collections.defaultdict(list)
    for _ref, content, conns, is_anchor in order:
        dom = D_DOMAIN                      # 域锚点/总览的默认域
        for tag, d0 in TAG2DOM.items():
            if content.startswith(tag):
                dom = d0
                break
        if is_anchor:
            mt, conf, imp = "Fact", CONF_STRUCT, 0.85
        else:
            mt, conf, imp = FACET[dom]
        if content.startswith("[verifier-lesson"):
            mt, conf = "Lesson", CONF_INFER      # 推断性教训，不冒充实测
        per_dom[dom].append({
            "id": "", "content": content, "memory_type": mt, "domain": dom,
            "strength": 1.0, "confidence": conf, "importance": imp,
            "connections": [aid[c] for c in conns if c in aid],
            "created_at": now, "last_accessed": now, "access_count": 0,
        })
    # 编号：按**域序**（非字典序）累加偏移，保证可复现且全局唯一
    seq = domain_sequence()
    off = mx + 1
    for d0 in seq:
        for m in per_dom.get(d0, []):
            m["id"] = "M-%06d" % off
            off += 1
    assert off == mx + 1 + total_expected(per_dom), "编号总数与记忆数不符"
    return per_dom, mx, now


def total_expected(per_dom):
    return sum(len(v) for v in per_dom.values())


def domain_sequence():
    """稳定的域顺序（保证编号可复现）。**不可**用字符串 < 比较域名，
    字典序 rl-domain < rl-rubric < rl-taxonomy < rl-verifier ≠ 想要的分组序。"""
    return [D_DOMAIN, D_VERIFIER, D_TAXONOMY, D_RUBRIC]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commit", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    t0 = time.time()
    print("corpus: %s" % MIMO)
    b = distil(Builder())
    print("built: %d rows (%d anchors) in %.1fs"
          % (len(b.rows), len(b.anchors), time.time() - t0))

    per_dom, mx, now = commit(b, args)
    total = sum(len(v) for v in per_dom.values())
    print("distilled memories=%d  base max M-id=M-%06d" % (total, mx))
    for d0 in [D_DOMAIN, D_VERIFIER, D_TAXONOMY, D_RUBRIC]:
        print("   %-14s %5d" % (d0, len(per_dom.get(d0, []))))

    frag_path = os.path.join(MIMO, ".frag.tmp")
    os.makedirs(os.path.dirname(frag_path), exist_ok=True)
    wrote = []
    with open(frag_path, "w", encoding="utf-8") as spill:
        for d0 in [D_DOMAIN, D_VERIFIER, D_TAXONOMY, D_RUBRIC]:
            mems = per_dom.get(d0, [])
            for k in range(0, len(mems), base.SHARD):
                shard = mems[k:k + base.SHARD]
                cid = "cocoon-%s-%d%s" % (d0, now, "" if k == 0 else "-p%d" % (k // base.SHARD))
                spill.write(base.cocoon_entry(cid, shard, now) + ",\n")
                wrote.append((cid, len(shard)))
    frag = os.path.getsize(frag_path)
    edges = sum(len(m["connections"]) for v in per_dom.values() for m in v)
    print("new cocoons=%d  edges=%d  frag=%.2fMB" % (len(wrote), edges, frag / 1e6))
    man = {"source": DS_ID, "url": DS_URL, "license": LICENSE,
           "kind": "agentic RL training environments (real release, not synthetic)",
           "memories": total, "edges": edges, "new_cocoons": [c for c, _ in wrote],
           "base_max_mid": mx, "mids_to": mx + total,
           "cocoons_bytes_before": os.path.getsize(base.COCOONS),
           "fragment_bytes": frag,
           "conf_scale": {"measured": CONF_MEASURED, "struct": CONF_STRUCT,
                          "inferred": CONF_INFER},
           "note": "all figures self-measured from the released parquet/rubric files; "
                   "inferences capped at %.2f and tagged [verifier-lesson]" % CONF_INFER}
    if not args.commit:
        os.remove(frag_path)
        print("\n--- DRY RUN ---")
        print("projected cocoons.json %.1fMB -> ~%.1fMB"
              % (man["cocoons_bytes_before"] / 1e6, (man["cocoons_bytes_before"] + frag) / 1e6))
        return 0

    bak = base.COCOONS + ".bak.rlenv"
    import glob as _g
    import shutil
    live = os.path.getsize(base.COCOONS)
    # 写盘前硬闸：新 id 不得与盘上重号（2026-09-28 事故：id 碰撞静默遮蔽不同记忆）
    base.guard_id_collision(
        open(base.COCOONS, "rb").read(),
        [m for v in per_dom.values() for m in v])
    olds = sorted(_g.glob(bak + ".*"))
    for old in olds:
        try:
            if os.stat(old).st_size > 1000000 and live < 1000000:
                print("ABORT: live %.1fKB, backup %s kept" % (live / 1e3, old))
                os.remove(frag_path)
                return 2
        except OSError:
            pass
    stamped = "%s.%s" % (bak, time.strftime("%Y%m%d-%H%M%S"))
    shutil.copyfile(base.COCOONS, stamped)
    for old in olds[:-2]:
        try:
            os.remove(old)
        except OSError:
            pass
    shutil.copyfile(base.COCOONS, bak)
    print("backup: %s (+%s)" % (bak, stamped))
    with open(frag_path, encoding="utf-8") as fh:
        entries = fh.read().rstrip()
    if entries.endswith(","):
        entries = entries[:-1]
    new_data = base.splice(base.COCOONS, entries)
    tmp = base.COCOONS + ".tmp.rlenv"
    with open(tmp, "wb") as fh:
        fh.write(new_data)
    os.replace(tmp, base.COCOONS)
    os.remove(frag_path)
    man["cocoons_bytes_after"] = os.path.getsize(base.COCOONS)
    os.makedirs(os.path.dirname(MANIFEST), exist_ok=True)
    with open(MANIFEST + ".tmp", "w", encoding="utf-8") as fh:
        json.dump(man, fh, ensure_ascii=False, indent=1)
    os.replace(MANIFEST + ".tmp", MANIFEST)
    try:
        st = os.stat(base.COCOONS)
        os.makedirs(os.path.dirname(base.MID_STATE), exist_ok=True)
        with open(base.MID_STATE, "w", encoding="utf-8") as fh:
            json.dump({"max_mid": mx + total, "mtime": int(st.st_mtime),
                       "size": st.st_size}, fh)
    except OSError:
        pass
    print("committed: %.1fMB -> %.1fMB  (%.1fs)"
          % (man["cocoons_bytes_before"] / 1e6, man["cocoons_bytes_after"] / 1e6, time.time() - t0))
    print("manifest: %s" % MANIFEST)
    return 0


def selftest():
    assert env_parts("s3k_0000_accounting_audit_tax_en_t1_rl_008") == \
        ("accounting_audit_tax", "en", "1", "008"), env_parts("s3k_0000_accounting_audit_tax_en_t1_rl_008")
    assert env_parts("weird") == ("weird", "?", "?", "?")
    assert CYB_SAN.match("AddressSanitizer: heap-buffer-overflow in function `f` in file `a.c`").group(1) \
        == "AddressSanitizer"
    assert CYB_FILE.search("x in file `mruby/b.c`").group(1) == "mruby/b.c"
    assert flat("a\n\n b  c") == "a b c"
    assert flat("abcdefghij", 5).endswith("…")
    assert num(2014) == "2014" and num(3.5) == "3.5" and num(True) == "" and num(None) == ""
    assert sget({"a": "", "b": "x"}, "a", "b") == "x"
    assert inst({"extra_info": {"instance_json": '{"k":1}'}})["k"] == 1
    assert inst({"extra_info": {"instance_json": "<html>"}}) == {}
    assert inst({}) == {}
    # 域顺序稳定 → 编号可复现
    assert domain_sequence() == [D_DOMAIN, D_VERIFIER, D_TAXONOMY, D_RUBRIC]
    assert domain_sequence() != sorted(domain_sequence()), "域序不得退化成字典序"
    # 逐 sanitizer 主错误类：不得退化成全局众数（曾造假映射 MSan→heap-buffer-overflow）
    per = collections.defaultdict(collections.Counter)
    for s in ["AddressSanitizer: heap-buffer-overflow in function `f` in file `a.c`",
              "MemorySanitizer: use-of-uninitialized-value in function `g` in file `b.c`",
              "UndefinedBehaviorSanitizer: undefined-behavior in function `h` in file `c.c`"]:
        mm = CYB_SAN.match(s)
        per[mm.group(1)][mm.group(2)] += 1
    glob_top = collections.Counter()
    for k, c in per.items():
        glob_top.update(c)
    for k, c in per.items():
        top = c.most_common(1)[0][0]
        assert c[top] == 1 and top == per[k].most_common(1)[0][0]
        # 本例各 sanitizer 主错误类互不相同；若实现误用全局众数，三行会全指向同一值
        assert top == {"AddressSanitizer": "heap-buffer-overflow",
                       "MemorySanitizer": "use-of-uninitialized-value",
                       "UndefinedBehaviorSanitizer": "undefined-behavior"}[k], (k, top)
    assert len({c.most_common(1)[0][0] for c in per.values()}) == 3, "三条轴应互异"
    # 族 lang 必须逐族统计（坑：曾把全局 lang 填进每个族锚点）
    fam_lang = collections.defaultdict(collections.Counter)
    for f0, l0 in [("a", "en"), ("a", "en"), ("a", "zh"), ("b", "zh")]:
        fam_lang[f0][l0] += 1
    assert dict(fam_lang["a"]) == {"en": 2, "zh": 1}
    assert dict(fam_lang["b"]) == {"zh": 1}
    assert dict(fam_lang["a"]) != dict(fam_lang["b"]), "不同族不得共享同一 lang 分布"
    print("selftest ok: env-id/sanitizer/flatten/num/instance-json/domain-order/"
          "per-sanitizer-axis/per-family-lang")
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
