# 吸收正典：Qwen-MM-Plugins → NeoTrix（2026-09-28）

> 来源：`https://github.com/QwenLM/Qwen-MM-Plugins`（main，2026-09-28 时点；
> core/search 均为 `plugin-versions.json: v1.1.0`）。**许可 Apache-2.0 ✅ 可 ship。**
> 方法：源码逐文件核实（tree 全量 1510 条目 → 按需 raw 拉取），所有主张带
> file:line 或实测输出；不可核实项显式标注。本轮用户决策：**A+B+C 全做**
> （会话客户端＋core/search 接线＋manifest 一致性门；mhs/video-spatio 只沉淀模式）。

---

## 1. 三个被旧文档带偏的前提（本轮实测纠正）

| # | 旧文档说法 | 2026-09-28 实测真相 |
|---|---|---|
| C1 | `McpRegistry` 在 `agent.rs:566` 的 `#[cfg(test)]` 块里（ABSORPTION-EXTERNAL §建议10） | **过时**。它在 `pub mod tool::mcp` 生产模块（`agent.rs:420-421/561-566`），`GLOBAL_MCP` 被 `entry/interactive.rs:110,344` 真实消费。但 `StdioNativeTool` 不会 MCP 会话协议（无 `initialize`，裸 `tools/call`，长驻服务器 30s 超时——`agent.rs:734-736` 自述）→ **调不通任何真 MCP 服务器**。本轮 P0 接线点 |
| C2 | `SearchResult` 7 份 | **8 处**（7×单数＋`nt_workspace.rs:158` 复数）；`skill_loader.rs:222` 那份此前无人记录 |
| C3 | 多模态是空白 | OCR（PaddleOCR ONNX＋回退链，`nt_world/ocr/mod.rs:128-281`）、语音转写（纯 Rust Mel 前端 568 行，onnx 门控）、PDF 嵌入图提取（lopdf 真解析）、视频帧提取（ffmpeg 双路真实现）**全是真的**。断的是最后一公里：`nt_io_multimodal_transform/mod.rs:3-9` 主动把 `ImageContent` strip 成文本（"目标模型始终不接触图像"） |

---

## 2. 缺口矩阵（全部 file:line/实测可复核）

| # | Qwen 能力 | NeoTrix 现状 | 判定 | 本轮动作 |
|---|---|---|---|---|
| 1 | `read_image` 动态分辨率 | 能发 `image_url`（`nt_http_engine.rs:298-305`），无 budget/scale 数学 | 🟡 PARTIAL | 接线（A2） |
| 2 | `media_info` 元数据优先 | ffprobe 零散调用，无统一工具/VFR 检测 | 🟡 PARTIAL | 接线（A2） |
| 3 | `read_video` 动态 fps | 帧提取得出（`nt_extract.rs:131-266`），送不到模型 | 🟡 断末公里 | 接线（A1 image 落盘→vision 路） |
| 4 | `visualize` 万物渲染成图 | 页渲染/3D/NIfTI/GIS/LaTeX 全无（`nifti\|stl\|gltf` 零命中） | 🔴 ABSENT | 接线（A2， notably NIfTI 页渲染） |
| 5 | `crop`/`draw_bbox`/`save_view` | 零对应 | 🔴 ABSENT | 接线（A2） |
| 6 | `api`: vision_chat/grounding/SAM3 | OCR 有；grounding/分割/云 VL 无 | 🟡/🔴 | 本轮不做（需 DashScope key，见 §5） |
| 7 | Omni diarization | ASR 有；说话人分离无 | 🟡 PARTIAL | 本轮不做（同上） |
| 8 | `search` + reverse-image | 只有路由（`search_router.rs`），执行层未见；反向图搜无 | 🔴 ABSENT | manifest 注册＋credential-gated（B） |
| 9 | video-memory / omni-memory | 只有转码管线，无分层记忆 | 🔴 ABSENT | 本轮不做（§5 路线图） |
| 10 | video-spatio | 零 3D | 🔴＋模式 | 只记模式（§4） |
| 11 | video-edit 生成 | 自家 stitcher 只产计划不执行（`video_stitcher.rs:427-429` 自述） | 🔴 stub | 本轮不做（§5） |
| 12 | blender/freecad | 需桌面应用 | ⛔ 不吸 | — |
| 13 | mhs 硬件 | 无硬件（全仓仅 1 处 Modbus 字符串） | 📐 只吸模式 | 只记模式（§4） |
| 14 | 打包模式（Skill＋MCP／不可变 tag／`check_manifests`／SYSTEM_DEPS） | 正对 4/3/4 注册表 chaos | 📐 模式 | 落地一半：一致性门（C）；不可变 tag 思想记入 §4 |
| 15 | MCP stdio 会话 | 见 C1 | 🔴 P0 | 落地（A1） |

---

## 3. 本轮交付（A+B+C）

### A1 会话客户端：`neotrix-core/src/nt_mcp_stdio_session.rs`（~700 行含 10 单测）

- per-call spawn 全握手：`initialize`(id=1) → `notifications/initialized` →
  `tools/list|tools/call`(id=2) → **stdin 保持打开等回包** → 关 stdin → 收尸。
- ⚠️ **实测证伪记录**：初版"写完三帧即关 stdin 再等退出"被真服务器证伪
  ——服务端收到 `CallToolRequest`（日志实证）但 stdin EOF 拆除会话，
  in-flight 请求被取消，零回包、rc=0。改写后才通。这是 R-SCAN-2 的活例子：
  脑内模拟（"EOF 即退出"）≠ 真实行为（"EOF 即拆会话"）。
- `image` 块不进返回字符串：base64 解码落盘 `artifacts_dir`，文本只留路径
  （接 neobot 现有 `image_url` vision 路）。失败降级不断链（文本如实记录）。
- 错误五类不折叠（`Spawn/Timeout/Protocol/Server/Io`），诊断带 stderr 尾＋stdout 头。
- 测试：bash 伪造服务器做真进程级 framing 验证（list/call-text/call-image/
  server-error/spawn-fail/timeout/dead-server/native-adapter＋2 纯函数），零 Python 依赖。

### A2 注册胶水

- `agent.rs` 最小 diff（~45 行）：`McpServer` 加 `use_session`＋`session_timeout_ms`；
  `register_stdio_session(+_global)`；`as_native_tools` 按标志分叉到
  `McpSessionTool`。旧 `StdioNativeTool` 路径零改动。
- `neotrix-core/src/nt_qwen_mm_manifests.rs`（~600 行含 9 单测）：core 7 工具＋
  search 3 工具的真实 manifest（工具名/参数形状逐项核对源码 `TOOL`＋Pydantic
  `Args`；描述为转述非逐字复制）；启动探测三阶（uvx→PATH→源码 checkout，
  零 spawn）；SYSTEM_DEPS 二进制探测；search key 存在性检查（值永不外露）；
  探测失败不注册、只给安装指引（fail-closed）；写文件三工具标 Medium。
- `entry/interactive.rs` 双路径接线（`run_interactive`＋ephemeral）：启动打印
  注册报告（未注册报原因＋hint，不断启动）。
- `lib.rs` 注册两模块（根目录，层门不扫根文件，零门风险）。

### A3 skill：`skills/nt_multimodal/SKILL.md`＋`index.json`

- 前matter（name/description/version/author/triggers）＋正文（media-first 纪律、
  budgets、save_view 链、confirm-before-commit、落盘 honesty、失败模式表）。
- `skill-gate`：我的条目零失败（strict 红的是 58 个存量无 license 条目，不含我）。

### B search

- manifest 已注册（`web_search{queries*}`／`web_extractor{urls*,goal*}`／
  `image_search{image_path*,bbox?,allow_public_upload=false}`——参数形状核对过
  `tools/*.py` 的 `Args` 模型）。credential-gated：无 key 不注册（实测本机无 key
  → 正确 skip）。配 key 即生产（launch 探测与 core 同链）。

### C 一致性门：`scripts/check-capability-manifests.sh`

- 抄 `check_manifests.py` 思想（manifest↔声明必须一致）＋本门既有风格
  （advisory 默认、`--strict` 供 CI）。M1 文件存在／M2 name 一致／M3 描述逐字
  一致／M4 name 唯一／M5 index 版本。
- 首跑：`checked=24 fails=19 warns=36`——**全是存量**（M3 双语漂移为主），
  我的条目零 mismatch。存量不清零前不接 CI（批量改描述是 churn，见门头注释）。

---

## 4. 只沉淀模式、不写码（用户决策）

### P1 mhs 安全模式 → shield 的下一课

固定 6 工具面（`discover/meta_info/read/write/health_check/reset`）＋三条硬纪律，
与 neobot fail-closed 哲学同构，值得抄的是**形状**而非代码：

1. **工具面不随硬件增长**——新设备只出现在 `discover` 里。映射：NeoTrix 加新
   执行器时，能力注册表应该是"新行"而不是"新类型"（对治 4/3/4 chaos 的同一种病）。
2. **host 侧硬限位不可绕过**：`confirm=true` 越不过 hard limit，"trying is not
   a plan"。映射：`nt_shield_enforcer` 的 irreversible 动作应有同等"确认也越不过"的
   硬地板（路线图 5.1"非不可宽化地板"的具体形状之一）。
3. **`estop` 免确认**：任何站在"你与停止之间"的东西都是 hazard。映射：取消/
   回滚路径（路线图 1.7）必须有一条免审批的快道。
4. **拒绝是信息**：超限被拒→上报，不找绕路。映射：审计日志应记录"被拒的尝试"
   为一等事件（路线图 5.3"以尝试为检测单元"的实例化）。

落点（未来）：`l3_embodiment/nt_shield_enforcer.rs:388-390` 的写操作注册表；
`nt_cancel.rs:29` 的取消语义。需 key 吗：否。需硬件吗：否（模式先行）。

### P2 video-spatio 感知/计算分离 → 推理管线的形状

"模型做感知（boxes/depth/motion 估计），无状态几何工具做数学（triangulate/
BEV/object_world_motion）"。落点（未来）：`nt_prediction_fusion.rs:632` 的帧采样
+ 几何计算分离——现在采样与判断焊在一起。另：`select_keyframes`
（uniform/motion/coverage/covisibility 四策略）可直接指导 `nt_extract.rs` 的采样策略。
需 GPU 吗：否（几何工具是 numpy/scipy，感知归 VLM）。

### P3 打包模式的另一半（本轮只做了一半）

Qwen 的 `plugin-versions.json`＋"Never move a published tag"＋`SYSTEM_DEPS`
表，本轮只落地了"manifest 一致性门"。剩余：给 NeoTrix 能力也立"不可变版本"
思想（与 `maturity_audit`＋capability-truth 门结合：`claimed>supported` 自动降级
时，版本号是诚实度的载体）。落点（未来）：`nt-core-capability-tree` 的版本字段。

---

## 5. 明确不吸收（反模式表）

| 来源 | 为什么不要 |
|---|---|
| `blender`/`freecad` 代码 | 需装桌面应用＋跑 live session；且 vendor 的 FreeCADMCP 是第三方 MIT（NOTICE 负担）。NeoTrix 零 3D，需求不存在 |
| `edu-agent` 全量 | 中文 K12 理科讲解视频＋Node/ffmpeg 重资产；方法论（skill-only 打包）已在 §4-P3 覆盖 |
| `omni-chatcut`/`omni-video2note`/`omni-skill-creator`/`omni-memory`/`video-memory`/`video-edit`/`api` 执行层 | 全系依赖 DashScope key（云 VL/Omni/生成）；无 key 不可验证，R-P79 拒绝"接了跑不起来的线"。`omni-skill-creator`（demo 视频→skill）对 120-skill 系统的方法论价值记入路线图候选，需 key 后另起轮次 |
| `mhs` 硬件执行 | 无硬件；只吸 §4-P1 模式 |
| 把 py 改写成 Rust | AGENTS.md 已定论：病根在接线不在语言。本轮零改写，全走会话桥接 |
| 存量 19 个 M3 描述漂移的"顺手修齐" | churn 零架构收益；按 skill 归属逐个认领 |

---

## 6. E2E 实证（2026-09-28，本机）

真服务器 `qwen-mm-plugins-core --version` → `1.1.0`；`--check-system` →
ffmpeg/ffprobe ✓（与 manifest 的 SYSTEM_DEPS 映射一致）。
sparse-checkout（48 py / 1.1M）＋venv（mcp 1.30／pydantic 2.13.5／pillow 11.3，
Python 3.14 兼容）。

| 用例 | 驱动 | 结果 |
|---|---|---|
| E2E-1 `read_image` | 与 Rust 客户端**同字节帧**的 Python 驱动（initialize→initialized→call，stdin 保持打开） | ✅ `600x800 → 448x576` 动态分辨率＋JPEG 块（b64 6216B） |
| E2E-2 `media_info` | 同上（自造 3s mp4） | ✅ 容器／编码／fps／帧数／无音频轨全量元数据 |
| E2E-3 `tools/list` | 同上 | ✅ 恰好 7 工具，与 `qwen_mm_core_tools()` manifest 逐名一致 |

⚠️ Rust 侧单测（`cargo test -p neotrix --lib nt_mcp_stdio_session / nt_qwen_mm_manifests`）
因内存门 BLOCKED 未跑（他窗 `cargo test -p` 占 98% CPU，见 handoff）。待跑命令：

```sh
sh scripts/ops/nt_mem_gate.sh; echo $?   # 非 0 不跑
cargo xl                                  # 最轻检查
cargo test -p neotrix --lib nt_mcp_stdio_session
cargo test -p neotrix --lib nt_qwen_mm_manifests
cargo test -p neotrix --lib agent::tool
python3 scripts/ops/nt_lock_audit.py neotrix-core/src   # 改了 .rs，必须重跑
bash scripts/check-layer-deps.sh --strict
```

---

## 7. 许可与署名

- Qwen-MM-Plugins：Apache-2.0（`LICENSE` 11358B 已核）。manifest 描述为转述；
  每工具注释标了源码出处；`skills/nt_multimodal` index 条目带 `license` 字段。
- 未 vendor 任何上游代码（会话桥接，无复制）。musl 注意：无。

## 8. 下轮入口

1. 跑 §6 待跑命令（mem 门开后第一件事）。
2. 配 `SERPER_API_KEY`（或 TAVILY/EXA/SERPLY）→ search 3 工具自动注册，e2e
   `web_search{"queries":["..."]}` 一轮。
3. 配 `QWEN_MM_PLUGINS_CHECKOUT`（sparse-checkout 目录）→ 无 uvx 环境也可生产。
4. §4-P1/P2/P3 进路线图（需先过 0.2 证伪门：每个模式先写"会削弱假设的结果"）。

## 9. 无 key 体的方案（2026-09-28 追加，用户问"无 key 有没有体的方案"）

> 结论：有。core 7 工具原生无 key（本地模式），search/api/omni 才要 key。
> 以下全部本机实测，无 key、零 Rust 改动（垫片走 `LaunchVia::Path`）。

| 件 | 位置 | 状态 |
|---|---|---|
| 一键 provision | `scripts/ops/nt_qwen_mm_setup.sh`（`setup\|check\|update`） | ✅ 本机跑通 |
| 落盘 | `~/.neotrix/qwen-mm/{src,venv,bin}`（仓外，prune 安全，删即卸载） | ✅ 48py＋venv（mcp 1.30/pydantic 2.13.5/pillow 11.3/pypdfium2/nibabel） |
| 任务索引 | `.neotrix/task-index.json` `qwen-mm-setup`（31→32 条） | ✅ |
| 接入 | `export PATH="$HOME/.neotrix/qwen-mm/bin:$PATH"`（或设 `QWEN_MM_PLUGINS_CHECKOUT`） | ✅ 自检 7 工具握手 |

无 key 实测矩阵（Python 驱动，与 Rust 客户端同字节帧）：

| 用例 | 结果 |
|---|---|
| `read_image`（800x600→448x576＋JPEG 块） | ✅ |
| `media_info`（ffprobe 全量） | ✅ |
| `read_video` fps=1（3 帧＋时间戳＋JPEG 块） | ✅ |
| `visualize` NIfTI（shape/dtype/spacing/orientation＋3 中心切片 JPEG） | ✅（此前全仓零 `.nii` 代码） |
| `visualize` Rust 源码（语法高亮文本） | ✅ |
| `tools/list` | ✅ 恰好 7 工具，与 manifest 逐名一致 |

Rust 活证明：`nt_qwen_mm_manifests.rs` 的 `#[ignore] live_qwen_core_keyless`
（list→media_info→read_image 落盘→read_video 全链，需 `NT_QWEN_MM_LIVE=1`＋
provision＋ffmpeg；缺前置 loud-fail 不静默过）。待 mem 门开后跑：
`NT_QWEN_MM_LIVE=1 cargo test -p neotrix --lib -- --ignored live_qwen_core_keyless --nocapture`

诚实边界：LibreOffice 本机缺 → Office 可视化降级（manifest availability 如实报）；
`visualize` LaTeX/HTML screenshot 未装对应后端（report-only，不阻塞）。
pre-commit 注意：`agent.rs`（存量非 `nt_` 名）若被 `git add` 会触发命名门
`exit 1`——提交本轮改动须 `--no-verify` 或给钩子加白（存量条件，非本轮引入）。
