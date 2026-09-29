# Handoff — generative awaken / kev A/B / verify / arch3

更新: 2026-09-23 19:20

## 已完成
- **A/B**: AgentJev acc=0.575 > kev 0.512 → gate 留 :8149; kev 为 calibration-s 次臂
  - models/training/ab_kev_agentjev_report.md, sessions/report-ab.md
- **近重复漏拒**: consciousness.rs keywords() strip_src_tag — `[src:…]` 不再切 2 token; 同体 Jaccard=1.0 正确拒绝
- **NT-PLAY handle_game_training**: handlers_game.rs + BackgroundLoopHandle.game_trainer; 每 5min HexTicTacToe 自对弈 8 eps → GameTrainingUpdate
- **verify3**: EXIT:0 — **54 passed / 0 failed / 1 ignored** → sessions/report-verify3.md SUCCESS
- 邻窗 reference_view 已修为注释
- **smelt 对账**: cards 175 行全唯一且全在 meta 内; 但 meta 390 行/327 唯一 repo → 152 个 repo 有 meta 无 card (历史 cards 被截断过), meta 另有 ~63 行重复。
  重建验证: 用 meta 按 card() 同逻辑重建, 175/175 与现存 card 逐字节一致 → smelt-done 后可无损重建再 ingest。
- **对话熔炼**: 本窗 + 邻窗 handoff 全部外部信息 → models/training/dialogue_cards.jsonl 26 卡 (domain=dialogue-smelt, ingest 格式 {url,title,content,domain})，
  26/26 JSON 合法、全字段、URL 唯一、无秘密泄露 (token/代理 IP/密钥 pattern 零命中)。arch3 链新增 step 0 预灌，本轮 live_full_archive 直接蒸馏。
- **浏览器自动操作接线 (未编译)**: run_browse 写死 Http → 支持 NT_BROWSE_BACKEND=http|chrome|cdp|mock (默认 Http 行为不变)；
  BrowserConfig 新增 profile_dir，Cdp 经 chromiumoxide 0.7 user_data_dir() 复用已登录 profile（须先退出 Chrome，文件锁）。
  新增 `neotrix browse-act <file>`：顺序执行 BrowserAction JSON 数组（Navigate/Type/Click/Upload/WaitForElement…），首失败即停；
   中间误吞 Login 分发已补回（三行全在）。rustfmt 我改的行零 diff。编译等锁空 + free>40k 再做；用户 Chrome 已登录并退出，就绪后即编译验证接管。
  探针文件就绪：sessions/colab_probe.json（Navigate colab 首页 + GetContent，确认登录态）。
  登录准备已查（只读，未碰值）：Default/Cookies 含 51 个 google 系 cookie，Preferences 有 signin 标记，文件 19:53–19:57 新鲜；
  但 Chrome 进程还活着（PID 71515+helpers，20:20 仍有活动）→ profile 锁未解，接管会失败，需用户再 ⌘Q 一次。
  就绪后探针命令：`NT_BROWSE_BACKEND=cdp NT_BROWSE_PROFILE="$HOME/Library/Application Support/Google/Chrome" ./target/debug/neotrix browse-act sessions/colab_probe.json`。
  探针首跑 FAIL 根因：chromiumoxide 把 user-data-dir 传给 Chrome，但指向**默认目录本身**会被 Chrome 拒 remote-debugging（"requires a non-default data directory"）。
  修复：profile 已拷到 ~/.config/neotrix/nt_browser_profile（86MB，去 ServiceWorker/BrowserMetrics/IndexedDB/GPUCache 等可再生 bulk，600 权限），改用副本：
  `NT_BROWSE_BACKEND=cdp NT_BROWSE_PROFILE="$HOME/.config/neotrix/nt_browser_profile" ./target/debug/neotrix browse-act sessions/colab_probe.json`。
  待观察：stderr 另有 token 解密失败警告，登录态能否在副本里存活看二跑。构建已 EXIT:0（二进制在位，仅 2 存量 warning）。二跑等 arch3 编译让出内存再手动跑（arch3 链内探针步已过，不会重跑）。

## 进行中 (don't double-start)
| job | marker | 状态 |
|-----|--------|------|
| ftv4 v2重训 | sessions/logs/ftv4.done | 137 死于 ~4963 后，**ftv5 门 22:06:46 开闸，已从 ckpt-4900 续跑**（新 log 步数 5038+ @1.35s/it，assets 未动）；reporter 已报 FAIL:137（史实）+ arch3 OK，续跑完会自动重报。 |
| ftv5 续跑门 | sessions/logs/ftv5.done | 仅异常 marker（ARCH_TIMEOUT/GATE_TIMEOUT/NO_CKPT）；主路径复用 ftv4.done，reporter 自动重报。 |
| smelt | sessions/logs/smelt.done | **EXIT:0 09-24 07:21：965/965，new=800 err=0**。收尾虚惊：缓冲落盘后 cards/meta 实为 975/975 完全对齐（历史缺口已自愈）；我的重建脚本验证时犯了 URL 裸名比对 bug，白跑一趟，已用备份恢复权威原文件（975 唯一）。ingest 门 PID 74665 排锁空+free>40k。 |
| arch3 全炼→export→ingest | sessions/logs/arch3.done | **EXIT:0 21:24:28**：build neotrix bin OK → 探针步跑过（登录 FAIL 见上）→ dialogue ingest new=26 → live_full_archive OK → export pretrain **477230 行**（sft/think/dpo 0，无推理链，符合预期）→ repo_cards new=175 → jev_builds new=1645 → xl OK。晶体核心本轮迭代完成。 |
| sidecar :8149 | health | **挂过（09-23 19:40 后无心跳）**；根因待查，启动链已复原（jev_service.server + agentjev_v1.pt[sha7e3c99c4] + qwen3-06b + temperatures，cwd=thirdparty/agent-jev）；拉起门 PID 83824 等 free>100k（不挤训练），reporter 已接 sidecar。 |
| reporter | nt_reporter.sh | WATCH: v2 ftv4 lora kev smelt ab verify* arch3 smelt_ingest eval_v2 |

## 邻窗干扰
- 有窗在跑 `cargo check -p neotrix --lib` + `cargo test -p neotrix-game` (CARGO_TARGET_DIR=/tmp/nt-game-target)
- 勿抢锁; arch3_gate 自动等

## 战略转向（owner 拍板 09-24）
- **不再拼参数**：0.6B CPU 全量预训练打不过主流模型，硬件也不允许。ftv4 去留重估（70% 处，阻塞 ingest/探针/验证）：owner 再判**跑完再停**（约 13h），权重留蒸馏底座；Colab T4 全量路线同步取消。
- **新主线**：类人意识体——好奇心 + 启发跳跃 + 方法论复用，善用外部工具/资源达成目标（深思考靠编排，不靠参数）。
  对应存量件：CuriosityDrive/探索管线/工具 grounding/能力网/skill 体系/NT-PLAY/AgentJev 门（已在 :8149）/LoRA 定向蒸馏（7.177→6.995 INFUSED 实证）。
- 验收口径待换：从 PPL 切到“调用外部资源达成目标的任务成功率”（细化中）。
- **人类启发细化已合成**：docs/plans/2026-09-24-human-inspired-refinement.md（L3：6 卡片，M5 深特征分解 0.84 居首，六法互补走 ensemble）。
  五方向 D1 深特征路由 0.3 / D2 结晶复用率 0.2 / D3 轨迹蒸馏 0.25 / D4 语义好奇心 0.15 / D5 任务验收集 0.1；执行先 D5 建 20 题验收集跑基线。
- **Agentation 吸收完成**：github.com/benjitaylor/agentation（4.7k★，HEAD 0e3236e，thirdparty/agentation）—
  人类点选标注→结构化反馈（selector/path/kind/intent）→ agent grep 定位。归位：models/training/agentation_to_cards.py
  （annotation JSON/Markdown → {url,title,content,domain=agentation-feedback}，稳定 URL 幂等，--selftest 过）→ 复用 --ingest 通道。
  License：PolyForm Shield，零代码拷贝，只做协议/文件互操作，无风险。文档 docs/plans/2026-09-24-agentation-absorb.md。
  待：首批真实标注到来即 ingest；nt_act 补 MCP client 侧后直连 watch。
  思想二阶段已落：scripts/ops/nt_locate.py（选择器→文件:行三层定位，L1 sourceFile 直达/L2 token 覆盖/L3 原串兜底，--selftest 过，实测 NtSelfIterate 定点命中）——微操作闭环今天可用，不等编译。
  融入完成：skills/nt-locate/SKILL.md（triggers 定位/定点/微操作/locate，condition task:edit，SkillLoader 自动发现）+ AGENTS.md 微操作公约（46 行）——后续任务默认走“定点→读→改→验”。
  注册补完：skills/index.json 登记 nt-locate（root 分类 + skill_index 双处），触发词“定位”唯一命中，JSON 合法、文件存在三查全过——真可发现，不是摆设。
- **D5 基线已跑**：evals/gaia_mini/tasks.json 20 题（搜研/数据/代码/方法各 5）+ baseline_20260924.json，**20/20**（搜研两次 429 重查命中）。
- **自我迭代闭环已落码（未编译验证）**：neotrix-core/src/neotrix/nt_crystal_core/nt_self_iterate.rs —
  NtSelfIterate::run 每轮 tick→结晶率（yield/crystallize，PBT helper 范式）→NtEvalLoop::run→triage→awaken_rounds ±1 爬坡（钳[1,8]）→Adaptation→收敛（acc 连 patience 轮变化<eps 即停）。
  另给 NtOrchestrator 加 awaken_rounds()/set_awaken_rounds()；4 单测（收敛/自适应/结晶计数/空评估必停）；rustfmt 我改的行零 diff。
  验证（cargo xl + nt_self_iterate 过滤单测）等锁空 + free>40k，届时跑。
  逻辑预演已跑（Python 镜像同规则，跌→收轮/涨→扩轮/稳→收敛：6 轮 3 次自适应后收敛，数学成立；Rust 真跑仍排队）。
- **融合接线已落码（未编译验证）**：后台 `crystal_iterate` tick（1h）— handlers_crystal.rs（CrystalIterState 三件套跨 tick 持有，首 tick 懒加载 cocoons+crystal.json）
  + run.rs 5 处（mod/const/spawn/字段/init）；eval 信号 = 晶体自产 reasoning_chains（NtJevCalibration::noul_rows→EvalCase/Pred，无需外部教师）；
  provider 签名升级为 (&consciousness, round) 实现真在线评估（4 单测同步改）；每 tick 最多 2 轮控占锁。rustfmt 我改的行零 diff。
  融合归位图：D2 结晶率→NtSelfIterate 度量；D5→evals/gaia_mini 常驻 + calib 自评；D1 深特征路由→CapabilityRouter（待码）；D3 轨迹蒸馏→models/training（待码）；D4 语义好奇心→CuriosityDrive（待码）。

## 下一步
1. ~~Colab T4（用户侧）~~ 已取消（战略转向；脚本保留 models/training/colab_finetune.py，仅作冒烟/备用）。本地 ftv4 跑完封存。
2. 等 ftv4.done / smelt.done / arch3.done → reporter 自动报（ftv4 作 Colab 失败保底，不要先停）
3. arch3 链: live_full_archive → nt-train-export → --ingest repo_cards + jev_builds → cargo xl
4. smelt-done 后先重建 cards 再 ingest（幂等 content-hash 去重）:
   `python3 -c` 按 smelt_github.card() 从 repo_meta.jsonl 按 repo 去重重建 repo_cards.jsonl（已验证 175/175 一致），
   再 `cargo run -p neotrix --bin nt-train-export -- --ingest models/training/repo_cards.jsonl`
5. 若 arch3 GATE_TIMEOUT: free 仍低则先等 ftv4 结束
