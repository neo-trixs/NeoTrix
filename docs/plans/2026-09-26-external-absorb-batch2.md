# 外部吸收 Batch2（2026-09-26，17 源）+ 缺陷 map + 核心进化建议

> 方法：双侦察窗只读研（READMEs + 1 论文），本窗收敛 verdict。凡 `不吸` 写明原因，凡 P0 给落地模块。

## 1. 源 verdict 表

| # | 源 | 一句话 | verdict | 落地 |
|---|---|---|---|---|
| 1 | code-yeongyu/oh-my-openagent | 多模型编码 agent 编排（11 agents/54 hooks/Team Mode） | **P0**（Hashline + Skill-MCP 按需拉起） | neobot 前端 / 能力树 |
| 2 | cocoindex-io/cocoindex-code | AST 语义代码搜索 CLI（增量索引，省 70% token） | P1（与 Ix 二选一） | SQLite 记忆 / 代码检索 |
| 3 | nadimtuhin/claude-token-optimizer | 启动 context 11k→0.8k（4 核心文件 + 懒加载） | **P0**（零成本） | neobot 启动 / 记忆预算 |
| 4 | markfulton/ai-employees | 文件夹即员工（ROLE/SCHEDULE/routines） | P1（业务模板） | routines / nt_shield |
| 5 | arxiv 2609.14011（ICL 收敛涌现） | paired-mapping ICL 六模态涌现 + 难度谱正相关 | P1（方法论） | nt_jev_calibration 评测 |
| 6 | browser-use/jev-ultrafast | 最快 web agent（原子 DOM 快照，无截图，单 trip） | **P0** | 浏览器内核 |
| 7 | ix-infrastructure/Ix | tree-sitter 系统图谱（explain/trace/impact 有界查询） | P1（重，需 Docker；与 coco 二选一） | 记忆检索 |
| 8 | t8y2/dbx | 25MB Rust+Tauri DB 客户端（桌面/CLI/MCP 一体） | P1（打包参照） | neobot 桌面 |
| 9 | XiaomiMiMo/verl（63★ fork） | 生产级 RL 后训练（GRPO/DAPO/tool-call） | P1（以上游 verl-project 为准） | 训练管线 |
| 10 | cactus-compute/needle | 8-29MB 端侧 2-bit 模型（calibrated confidence 头） | **P0** | JEV 校准 / tool-call 约束解码 |
| 11 | google/artemis | 自然语言真机 Android 自动化（Flash/Pro 双 profile） | P1 | 调度 / 元素定位 |
| 12 | bendlang/bend | 并行语言 + LAWS 可证明 AI 约束 | P2（思想） | skill 验收门 |
| 13 | win4r/MuseAI-Skills | 68 skill 存档（skill-creator 范式 + manifest + eval） | **P0** | skill 系统 |
| 14 | CopilotKit/openmuse | 个人 agent（durable task + SQL lease + stored review） | **P0** | neobot / 能力树 |
| 15 | s1dashu/ip-as-logo-skill | 单文件极简 skill（批量候选范式） | P1 | skill 微模板 |
| 16 | stanfordnlp/dspy | programming-not-prompting（GEPA/Assertions） | **P0** | 训练管线 / skill prompt 进化 |
| 17 | AnmolSaini16/mapcn | React 地图组件（shadcn 兼容） | P2（无地图刚需；CARTO 商用需授权） | — |

## 2. 缺陷 map（本次暴露 + 吸收反推）

| 缺陷 | 证据 | 等级 | 对应模块 |
|---|---|---|---|
| 第三方 provider `/models` 不可达 → 池子零行 → 对话框切不过去 | 本窗实测（nvapi 活着才没爆；端点一挂即失联） | P0 ✅ 已修 | `nt_cmd_sys::neobot_models` + CLI `cmd_models` fallback 行 |
| 配对晶体后第三方池子被晶体面板整面盖住 | `paintCrystalPanel` return 前无池子分支 | P0 ✅ 已修 | `main.ts` 晶体面板追加池子组（现拉，不依赖 cache） |
| `cmd_prune --force` 双删（execute 已删 + 又删一次 → NotFound 裸奔致 save 流产） | 本窗 prune 验证节点复现 | P0 ✅ 已修 | 能力树 `cli.rs`（NotFound 视为已完成） |
| `.neotrix/capability_registry.json` 仍是 v1.0.0 domains 形 → bud `missing field nodes` | 本窗实测复现 | P0 ✅ 已修（`migrate_legacy` + `.bak-legacy` + 单测） | 能力树 `registry.rs`/`cli.rs` |
| 用户把 nvapi **明文 key 当 key_env 存进 SQLite**（`provider list` 直接打印） | 本窗 `provider list` 目击 | P0 待用户 | 需用户：`export nvapi_KEY=<key>` 后 `provider add --key-env nvapi_KEY` 重写；另 `provider list` 应脱敏 key_env 回显 |
| skill 无权限 manifest + 无行为 eval（坏 skill 全权裸奔） | MuseAI-Skills 对照 | P0 未动 | skill 系统（`manifest.yaml` + `eval/scenarios.yaml` 三件套） |
| tool-call JSON 解析靠运气（无 grammar 约束） | needle 对照 | P0 未动 | 浏览器内核 / skill 执行器（约束解码或 schema 校验门） |
| 启动 context 无预算（全量注入） | token-optimizer 对照（11k→0.8k） | P0 未动 | neobot 启动（4 核心 + 关键词懒注入 + `audit`） |
| 浏览器内核无原子 DOM 快照/元素编号表（截图流，高 token） | jev-ultrafast 对照 | P0 未动 | 浏览器内核执行器 |
| 任务中断即丢（无 durable task/SQL lease） | openmuse 对照 | P0 未动 | neobot（pause/resume/retry + receipts） |
| skill prompt 靠手调（无进化器） | dspy 对照 | P1 未动 | 训练管线（GEPA/teleprompter 排期） |
| 代码检索纯 grep（无 AST/增量索引） | coco/Ix 对照（二选一） | P1 未动 | 记忆检索 |
| RL 管线缺失（只有 SFT/LoRA/DPO） | verl 对照 | P1 未动 | 训练管线（GRPO 排期，以上游为准） |
| T46（`Result<_,String>` 扩散） | 9-26 判定 stale | 关闭 | 重定为新代码门禁，不管存量 |

## 3. 核心进化建议（按序，最多 5 条）

1. **JEV 置信头外置化**（needle P0-1）：Platt 分桶是后验校准，缺 act/confirm/refuse 在线路由。
   在 `decide_with_floor` 后加三档门（conf ≥ 高 → act；中 → confirm；低 → refuse/escalate），
   阈值按域桶自适应（高风险域门已 0.85，直接复用）。
2. **浏览器内核原子化**（jev-ultrafast P0-1）：DOM 快照 → 元素编号表 → 8 操作受限头 →
   单 trip op/target → 遮挡重检 + 200ms 等待策略。token 降一个数量级，E2E 方差首先降这里。
3. **durable task**（openmuse P0-5）：routine/task 加 pause/resume/retry + receipts（SQLite 表），
   窗口关闭不再丢任务；stored review（不确定外部写先存后示，无隐藏重试）进满意度门控。
4. **skill 三件套**（MuseAI-Skills P0-3）：瘦主文件 + `manifest.yaml` 方法级权限 +
   `eval/scenarios.yaml` 行为验收；新 skill bud 门强制三件套，老 skill 分批补。
5. **启动 token 预算**（token-optimizer P0-4）：4 核心文件 + `.claudeignore` + 关键词懒注入 +
   `audit` 健康检查；目标冷启动 <1k token（现无预算，先量后砍）。

## 4. 本窗落地记录（neobot 模型管理修）

- `neobot_models`（桌面 IPC）+ `cmd_models`（CLI）：`/models` 不可达但配了 model →
  fallback 行（`valid_model_id` 同门，`pub` 化复用），发现失败不挡已配置项。
- `main.ts paintCrystalPanel`：配对后追加“第三方池子”组（现拉 `neobot_models`，去重，高亮沿用），
  点击走统一 `{provider, model}` 存 + `resolve_model_engine`（`get_provider` + `http_engine(override)`）。
- 验证：隔离 DB（NEOBOT_DATA_DIR=/tmp）加不可达端点 → fallback 行出现；`tsc` 零错；
  `cargo test -p neotrix-neobot --lib` 76 passed；用户库零写入（`provider list` 复核无 verifyprov）。
- ⚠️ 目击用户 nvapi key 明文落库（key_env 字段存的是 key 本体）：待用户给环境变量名后重写；
  另建议 `provider list` 脱敏回显（未动，等用户确认）。
