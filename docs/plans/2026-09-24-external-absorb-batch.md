# 外部批量吸收报告（2026-09-24，40 源去重）

> 方法：`worldbuilding-absorption`（HIGH=五阶段完整，MEDIUM=公理+行动，LOW=P1 一行）。
> 证据：各 README 已读（URL 见表）；impl 深挖随接线单做。R-P79 接线 = 本报告 + §7 接线单（代码实现等内存窗）。

## P0 深吸 · HIGH（5）

### 1. browser-use/jev-ultrafast（19.2k★）——NT-BROWSE
- 一句话：动态索引动作空间的浏览器 agent，单 goal 跑通订票 7.1s。
- 公理：①运转=观测→元素表→单 TypeSafe 请求（操作头+目标头推测扇出）；②稀缺=网络往返+上下文（无截图环、结构化状态、可视文本才进）；③金手指=小 LLM 只写 TYPE_TEXT。
- 力量体系：原子 DOM 快照（freshness+遮挡拒识）→ 有界等待（200ms/2帧/50ms）→ JSON 门禁打字 → DONE 独立验证。
- 接线：`nt_io_browser_engine.rs` 加 4 件——索引元素表、单请求推测扇出、打字 JSON 门、DONE 验证器。对标探针 2/2 的下一步。
- 行动：HIGH，D-BROWSE 增强（内存窗后）。

### 2. leechen298/Code2Skill（8★，小而利）——D2
- 一句话：从源码真实调用点炼出 Functions + MCP + Skills + 离线测试。
- 公理：①运转=调用点即能力源（无调用不炼）；②稀缺=业务语义（review-flow/review-source 双审守）；③金手指=离线验证默认（不碰 live API）。
- 接线：D2 helper 合成管线（repo_cards→调用点→Skill 包），直填 skills_index=0 的坑。
- 行动：HIGH，D2 实现并入。
- **学术本体（2026-09-24 重试补获）**：`arxiv 2609.05571`《Grounded Skill Synthesis from Code at Scale》
  （Tong et al., 2026-09-04, cs.SE/CL）：19,769 仓库 → **CodeSkillBank 1,006,822 条接受记录**；
  验证法 = **source-body-blind reconstruction + source-aware comparison**（盲重建+源码感知对比——
  直接采用为 D2 skill 验收门）；72 场评测平均 **+11.7%**，57/72 胜；同接口下 repo 派银行
  **7/7 全胜 trajectory 派**（经验不足时仓库知识优先——证明 D2 先于 D3 的排序正确）；
  AI 生成代码炼出的 skill 通过率 93.5% ≈ 人写 93.0%（AI 代码也可炼，供给无限）。
  证据：`export.arxiv.org/api/query?id_list=2609.05571`。

### 3. vectorize-io/hindsight（26.6k★）——晶体记忆
- 一句话：learn-not-remember 的 agent 记忆系统（LongMemEval SOTA）。
- 公理：①运转=retain/recall/reflect 三算子；②稀缺=注意tokens（mental models 预写常读）；③金手指=observations（多记忆合并为带证据+proof count 的信念，只 refine 不覆盖）。
- 接线：① observations 合并 → 修 insights=0；② mental models 常问问题 → 开机页；③ 4 路召回+RRF+rerank → 检索升级；④ memory defense → secrets 表执行化；⑤ per-repo bank from git → repo_cards 建 bank。
- 行动：HIGH，分 5 单独立排期（①②优先）。

### 4. trailhq/Graft（9.1k★）——D1/定位
- 一句话：无 embedding 的 markdown 节点图 + tree-sitter 布线，SWE-bench +12pts。
- 公理：①运转=含义内联（节点自带答案，免二次读）；②稀缺=token（crux 存代码文本不用行号，防漂移）；③金手指=3ms 新鲜度（working tree 即问即刷）。
- 接线：① `nt_locate.py` 升级（crux-as-text）；② push/pull 双模 → D1 路由（快用 push，准用 pull，pull 98%）；③ corrections→rules → experience-tree。
- 行动：HIGH，D1 实现并入（与深特征表正交，可同单）。

### 5. repowise-dev/repowise（7k★，AGPL）——审计/死码/文档
- 一句话：零 LLM 调用的确定性五层（graph/git/health/dead-code/doc-drift）。
- 公理：①运转=可计算先行（能算的不问模型）；②稀缺=评审预算（defect lift/费用实证）；③金手指=任务形 MCP（一次给全，`distill` 可逆压缩）。
- 接线：① dead-code 检测 → 邻窗清理（要规则书）；② doc-drift → DOCUMENTATION-MAP 自动核；③ impacted-tests → cargo 只跑受累单测；④ distill → 日志压缩（ftv4.log 3.4MB 教训）；⑤ transcript 挖 decisions → D2。
- 行动：HIGH，①②④优先（零模型，内存窗外可先行其 Python 版思想：手写 distill.py）。

## P0 深吸 · MEDIUM（公理+行动，10）

| 源 | 一句话公理 | 接线/行动 |
|---|---|---|
| addyosmani/agent-skills（98.7k★） | 流程非散文/反合理化表/验证不可协商/渐进披露；25 skills + evals/ 路由评测 | skills 格式升级（anatomy+evals/）；D1 对照组即其 evals 翻版。HIGH |
| elizaOS/eliza（19.5k★） | Plugin=actions/providers/**evaluators**/services；按能力路由 local/direct/Cloud；scenario runner | evaluator 组件补齐；per-capability 路由对标 Auto Exacto。MEDIUM |
| stanford-oval/storm（31.5k★） | 视角引导提问+writer↔expert 模拟对话；便宜模型跑量、强模型收尾；Co-STORM 心智图 | 提问质量→D4；心智图→domain_clusters；VectorRM→cocoons 接地。HIGH（提问术） |
| Yinsongxu/LLM2Jev（282★） | prefill-only 评分 + 候选独立 + 共享前缀复用（SGLang Radix/MLX） | NLL_select 提速（prefix 复用）；D4 judge 降本。HIGH |
| hr98w/jev-visual（261★） | 共享上下文→fork→批量后缀→代码组装；任务简化方法论（分类代跟踪） | NT-PLAY 视觉；D5 出题简化术。MEDIUM |
| autonomous-ai/openharness（797★） | DSH=harness.json 文件夹（指令+skills+工具链+检查+viewer），app 零改动；不包装 agent | skill 包格式参考；NT-PLAY 房制。MEDIUM |
| anthropics/code-migration-kit（官方） | judge-first（无裁判不开工）；rulebook 一次决策；depmap 确定性；bakeoff；queue 由磁盘定（停=免费）；修流程不修代码 | 邻窗死码清理/任何重构先立裁判+规则书；D5 纪律。HIGH（流程件，零码） |
| google/ax（9.3k★） | Task/Workspace/Gateway/Model 四原语；Gateway 用 allowlist（比我方 denylist 强）；suspend/resume 检查点 | allowlist 制升级 denylist；suspend/resume  formalize（ftv6 模式已验证）。MEDIUM |
| Google Code Wiki（codewiki.google/，官方产品） | PR-merge 钩子重生文档（docs 随代码进化）；wiki 即 chat 知识库；段段超链回代码；自动架构图 | DOCUMENTATION-MAP 接 PR 钩子（我方文档更新全靠手）；与 repowise doc-drift 同单。HIGH（机制件） |
| FSoft-AI4Code/CodeWiki（ACL 2026 论文+开源） | 分层分解（DP 思想保架构上下文）+ 递归 agent 动态委派 + 多模态合成；68.79% 胜 DeepWiki 64.06%；CodeWikiBench | 大仓文档生成法；委派判据（圈复杂度/语义多样性/窗口占用）可抄。MEDIUM |
| PorunC/CodeWiki（AST+GraphRAG+MCP） | Lite SQLite 本地索引；agent 写页四件套 plan/evidence/save/validate（引用必验才有效） | validate 门即我方证据纪律；MCP 工具面参考。MEDIUM |
| agent-sh/agentsys（988★） | detection（确定性）vs judgment（LLM）；certainty 分级；banthis 负记忆；deslop；agnix（423 规则配制 lint）；pipeline>模型档 | banthis（负经验！我方只有正）；agnix lint 我方 skills/；drift-detect 对 handoff。HIGH（banthis+agnix） |
| rahulnyk/knowledge_graph（4.1k★） | **概念非实体**（"宜人天气"比"北京"更有意义边）；共现 W2+语义 W1；本地 Ollama 零 GPT | graph_cache/domain 聚类改 concept 粒度。MEDIUM |

## P1 简吸（一行，19）

| 源 | 一句话 | 去向 |
|---|---|---|
| semantica-agi/semantica 13.4k★ | 图原生上下文基建 | KB 图层对标（后续深吸候选） |
| block/buzz 34k★ Rust | 蜂巢思维通讯层 | 通讯层参考 |
| msitarzewski/agency-agents 154k★ | Shell 版 agent 百宝箱 | 采单条技巧，不整体引 |
| SenteLabsAI/OpenExecutive 5.2k★ | 8 专家单人格高管团 | meta-shell 人格参考 |
| FxEmbed 5.2k★ | X 嵌入修复 | embed 管线参考 |
| momenbasel/PureMac 6.7k★ Swift | 零遥测 mac 清理 | reporter/系统位参考 |
| CyberStrike 2.8k★ | AI 进攻安全 harness | nt_shield 对抗参考 |
| muellerberndt/cadence 252★ | 共识均衡世界模型 | WorldModelV2 对标（深吸候选） |
| yetone/kill-ai-slop 1.2k★ | AI 味扫描 skill | des Anti-Slop 同盟 |
| imxv/Pretty-mermaid 1.4k★ | 15 主题 mermaid | fireworks-tech-graph 素材 |
| fuxiaoai/tidings-rss 456★ | 活验证 OPML 情报源 | daily-intel  handler 供源 |
| anmolkapil/plexo 923★ | 多连接并行下载 | prefetch 提速参考 |
| whitelonng/mancode 361★ | 防过度工程五模式 | deslop 规则参考 |
| yibie/awesome-autoresearch 761★ | autoresearch 列表 | autoresearch skill 供源 |
| CopilotKit/OpenMuse 1.8k★ | 个人 agent（浏览器+终端） | AG-UI 协议参考 |
| unreallabsai/unreal-agent 1.7k★ Go | 异步优先 harness | 异步模式参考 |
| repowise类比略 | — | — |
| veloren（Rust 体素 RPG，7.5k★） | Dwarf Fortress 式体素世界 | NT-PLAY 世界架构参考（Rust 同源） |
| osgameclones 3.1k★ / opensourcegames / PixelSRPG-Forge 242★ / art_resources | 游戏克隆谱系+像素素材 | NT-PLAY 世界构建/素材（novel_queue 同源） |

## P2 跳过（7，记原因）

benjitaylor/agentation（已吸→nt-locate）/ sindresorhus/awesome（纯列表无机制）/
topics/game-assets（列表页）/ libregamewiki（首页无机制）/
arxiv 2609.05571（已补获，见 Code2Skill 卡学术本体；P2 除名）/
重复项（OpenMuse/unreal-agent/hindsight 各×2，已去重）/
sarthakagrawal927/storagedaddy 1★（Swift 磁盘工具，相关度低）

## §7 路线图缺陷补齐矩阵（本次吸收的净增益）

| 缺陷 | 补法（源） | 落点 |
|---|---|---|
| NT-BROWSE 无动作空间理论 | 索引表+推测扇出+JSON 门+DONE 验证（jev-ultrafast） | browser_engine |
| skills_index=0、无合成管线 | 调用点炼 skill（Code2Skill）+ 节点含义内联（Graft）+ anatomy+evals（agent-skills） | D1/D2 |
| insights=0、无合并机制 | observations 带证据合并（hindsight） | D2/晶体 |
| 召回只有 FTS+向量 | 4 路+RRF+rerank（hindsight）+ concept 粒度（knowledge_graph） | 检索 |
| 无负记忆 | banthis（agentsys） | experience |
| 无配制 lint | agnix 423 规则（agentsys） | skills/ |
| 计划与代码漂移无人查 | drift-detect（agentsys）+ doc-drift（repowise） | handoff/DOCUMENTATION-MAP |
| 死码清理无章法 | rulebook+depmap+judge-first（migration-kit）+ dead-code（repowise） | 邻窗协同 |
| NLL/judge 太贵 | prefill-only+前缀复用（LLM2Jev）+ 共享批量（jev-visual） | D3/D4 |
| 提问质量低 | 视角引导+模拟对话（STORM）+ 心智图（domain_clusters） | D4/知识管线 |
| 漏：sandbox/allowlist | Gateway allowlist + suspend/resume（ax） | shield/门控 |
| 漏：跨机多 agent | DSH/relay（openharness）/ elizaOS per-capability 路由 | 远期 |

## R-P79 接线单（本 session 已接线：报告+索引；代码实现排内存窗）

- [x] 本报告落盘（吸收交付物）+ handoff §9 索引
- [ ] D1 单：Graft 节点格式 + agent-skills evals（与深特征表同单实现）
- [ ] D2 单：Code2Skill 合成 + hindsight observations + banthis 负记忆
- [ ] 检索单：4 路召回 + concept 粒度（独立小单）
- [ ] 流程件（零码即用）：migration-kit 裁判优先 + agentsys drift-detect（下次重构/邻窗清理前读一遍 prompts/00）
- [ ] 手写 distill.py（repowise 思想，纯 Python，轻活可插队）
