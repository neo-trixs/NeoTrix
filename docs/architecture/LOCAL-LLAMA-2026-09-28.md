# 本地模型（llama.cpp）— 2026-09-28

> 从 `AGENTS.md` 迁出（该文件有 130 行硬门限，R-P199 口径）。
> **信息一字未减**，只是把"怎么用"从"必须读"降级为"要用时读"。

## 本地模型（llama.cpp）— 2026-09-28

**权重位置**：`<repo>/models/`（12,224,467,808 B / 157 文件，已 SHA-256 抽验）
＋ 归档区 `~/Downloads/Neo/neotrix-archive/models/`（字节完全相同，兜底副本）。
两者都被 `.gitignore:27` 的 `models/*` 覆盖 ⇒ **git 保护不到，删了只能重下**。

**搜索路径真源**：`neotrix-core/.../llama/llama_process.rs::model_search_dirs()`
（2026-09-28 起取代此前散在两处的硬编码目录表 + `/Users/neo/...` 字面量）。
优先级：`NEOTRIX_MODEL_DIRS`(`:` 分隔) → `CARGO_MANIFEST_DIR/../models` → `models`(cwd)
→ `$HOME/.cache/neotrix/models` → `$HOME/.ollama/models{,/blobs}`。
⚠️ **项目不读 `.env`**；且 `~/.zshrc` 只被**交互式** shell 读取（实测
`zsh -lc` 拿不到、`zsh -ic` 才拿到），所以变量写在 **`~/.zshenv`**。
再上一层：桌面 App 由 launchd 启动，连 `.zshenv` 也读不到 —— **那条路径不依赖本变量**，
权重放在 `<repo>/models/` 就会被 `CARGO_MANIFEST_DIR` 推导到，零配置。

**启动 flag 的三个坑**（漏任一条就会出现「装完开不了话」）：

| flag | 漏了会怎样 |
|---|---|
| `--jinja` | 退回启发式解析器，不认 `<tool_call>` XML 与 `<think>` 分隔符 → 工具调用以纯文本漏出 |
| `--reasoning off` | 关 thinking 的**唯一**可靠方式。`--chat-template-kwargs '{"enable_thinking":false}'` 已被**静默忽略**；`--reasoning-budget 0` 反而在每个响应开头造出 `</think>` |
| `--ctx-size` **显式给** | 省略时 llama.cpp 按 metadata 尝试分配满原生窗口（Qwen3.5 = 262144）→ 16GB 机 OOM |
| `--no-prefill-assistant` | 否则 agent 循环撞 `400 "Assistant response prefill is incompatible with enable_thinking."` |

判定 `--reasoning off` 真的生效：启动日志须出现 `init: chat template, thinking = 0`。

**ctx 分档按架构而非按文件大小**（`compute_optimal_config`）：Qwen3.5/3.6/3.8 是
Gated DeltaNet + Gated Attention 混合架构，`full_attention_interval=4` ⇒ 32 层里只有
8 层是真注意力，其余是**不随上下文增长**的固定状态。KV/token = 8×2×4×160×dtype；
`q4_0` 下 **5.6 KB/token ⇒ 262144 tok 只要 1.47 GB**（同尺寸纯 Transformer 4B @f16
要 16.8 GB，单机装不下）。故混合架构 ≤5GB 给满 262144、9B 级给 131072（thinking
官方底线），纯 Transformer 保持 4096/8192/2048 不动。

**已实测**（2026-09-28，M5 16GB，llama.cpp 0.3.0 build 10621，
`HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-Q6_K`）：
28.8 tok/s gen · 243–348 tok/s prefill · RSS 6.6–7.2 GB · 59,307 tok 长上下文
检索正确（问"多少段落/最后编号"答"700/699"）· 工具调用 `finish: tool_calls` 且
args JSON 合法 · 多轮 agent 循环无 400 · 无 `<think>` 泄漏。

---

## 复算

```bash
grep -m1 '^version' crates/neotrix-neobot/Cargo.toml
find crates/neotrix-neobot/src -name 'nt_llama.rs' -exec wc -l {} \;
rg -c '#\[tauri::command\]' /Users/neo/Downloads/Neo/neobot/apps/neobot-desktop/src/ | awk -F: '{s+=$2} END{print s}'
```

## 关联

- 桌面 App 已统一到 `~/Downloads/Neo/neobot`（独立可运行）。本仓的
  `crates/neotrix-neobot` 降为**库**，其 `nt_llama` 是 CLI 与桌面端共用的
  **唯一**本地推理实现（`neotrix-core/.../llama/mod.rs` 只是 re-export）。
- 历史教训见 `docs/architecture/VALUE-EXTRACTION-SRC-TAURI-2026-09-28.md`：
  归档的 `src-tauri` 曾因不想依赖 214 依赖的 neotrix-core 而**复制**了一份
  进程启动，副本缺 `--jinja` 且 ctx 写死 4096，即"装完开不了话"的根因。
  **复制实现是 bug 的来源。**
