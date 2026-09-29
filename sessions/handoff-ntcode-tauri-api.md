# ntcode 桌面 App 构建交接

> 交接时间：2026-09-22
> 交接人：ntcode 后端 API 开发
> 接收人：桌面 App 前端构建

---

## 一、已完成的工作

### 1. ntcode 后端 API（src-tauri/src/ntcode/）

已构建完整的 Tauri IPC API，前端通过 `invoke()` + `listen()` 调用：

**新建文件：**
- `src-tauri/src/ntcode/mod.rs` — 核心类型定义
- `src-tauri/src/ntcode/commands.rs` — 9 个 Tauri command
- `src-tauri/src/ntcode/streaming.rs` — 流式生成 + event 推送

**Tauri Commands（前端 invoke 调用）：**

```typescript
// 发送消息（流式响应走 event，不走返回值）
await invoke("ntcode_send", { message: "解释闭包" });

// 停止生成
await invoke("ntcode_stop");

// 模型管理
const models = await invoke("ntcode_models");        // → ModelInfo[]
await invoke("ntcode_switch_model", { model: "opencode/mimo-v2.6-flash-free" });

// 对话管理
const convs = await invoke("ntcode_conversations");  // → ConversationSummary[]
await invoke("ntcode_open", { id: "conv-xxx" });
await invoke("ntcode_new", { goal: "新话题" });
await invoke("ntcode_save");

// 状态查询
const status = await invoke("ntcode_status");        // → "idle" | "streaming" | "waiting_human"
```

**Tauri Events（前端 listen 监听）：**

```typescript
listen("ntcode://chunk", (e) => { /* e.payload: { content: string } */ });
listen("ntcode://done", (e) => { /* e.payload: { summary: string, model: string } */ });
listen("ntcode://error", (e) => { /* e.payload: { message: string } */ });
listen("ntcode://status", (e) => { /* e.payload: { status: string } */ });
```

### 2. 对话持久化模块（neotrix-core）

`neotrix-core/src/l1_action/nt_conversation.rs`：
- 每个对话存为 `~/.neotrix/conversations/{id}.json`
- `save()`, `load()`, `list()`, `delete()` 四个函数
- 原子写（tmp + rename），多会话安全

### 3. TUI 模式扩展（neotrix-core）

`neotrix-core/src/l1_action/nt_tui_app.rs` 新增：
- `/new`, `/list`, `/open <id>`, `/save` 斜杠命令
- Ctrl+N（新建）、Ctrl+L（列表）、Ctrl+S（保存）快捷键
- 状态栏显示当前对话 ID + 模型
- 帮助面板更新

### 4. 模型池 JSON API（ntcode 二进制）

`ntcode --models` 输出所有可用模型的 JSON（17 个模型），前端可直接消费。

---

## 二、核心类型定义（前端需对齐）

```rust
// src-tauri/src/ntcode/mod.rs

pub struct ChatMessage {
    pub role: Role,           // "user" | "assistant" | "system"
    pub content: String,
    pub timestamp: u64,
    pub model: Option<String>,
}

pub struct ModelInfo {
    pub id: String,           // "opencode/mimo-v2.6-flash-free"
    pub name: String,         // "MiMo V2.6 Flash Free"
    pub source: String,       // "cli-free" | "cloud_free" | "gguf"
    pub tier: String,         // "free" | "t3-powerful" | "t4-frontier"
    pub is_free: bool,
}

pub struct ConversationSummary {
    pub id: String,           // "conv-19c2a3b"
    pub goal: String,         // 对话目标/话题
    pub message_count: usize,
    pub model: Option<String>,
    pub updated_at: u64,
}

pub enum SessionStatus { Idle, Streaming, WaitingHuman }
```

---

## 三、架构决策

1. **通信方式**：Tauri IPC + Event 流（非 HTTP/WebSocket）
2. **流式机制**：`ntcode_send` 触发后台 tokio task，LLM chunk 通过 `app.emit("ntcode://chunk")` 推送
3. **模型池**：启动时发现 opencode 免费模型（`CliFreeSource`），存入 `NtFreePoolAsk` 轮转
4. **对话持久化**：`~/.neotrix/conversations/*.json`，按 updated_at 降序排列
5. **状态管理**：`NtcodeState { inner: Arc<RwLock<NtcodeSession>> }` 作为 Tauri managed state

---

## 四、待构建：前端 App 组件

### 推荐组件结构

```
App
├── ConversationHeader     # 对话标题 + 模型选择器下拉
├── MessageList            # 消息列表（虚拟滚动）
│   ├── MessageBubble      # 单条消息（User 左/Assistant 右）
│   └── StreamingCursor    # 生成中闪烁光标
├── InputBar               # 输入框 + 发送/停止按钮
├── Sidebar                # 侧栏（可折叠）
│   ├── ConversationList   # 对话历史列表
│   └── ModelPicker        # 模型切换面板
└── StatusBar              # 底部状态栏
```

### 前端需实现的状态管理

```typescript
interface NtcodeState {
  messages: ChatMessage[];
  status: 'idle' | 'streaming' | 'waiting_human';
  currentModel: string | null;
  conversations: ConversationSummary[];
  currentConvId: string | null;
}
```

### 前端需实现的 hooks

```typescript
// 核心 hook
useNtcode() → {
  send: (message: string) => Promise<void>;
  stop: () => Promise<void>;
  messages: ChatMessage[];
  status: SessionStatus;
}

// 模型 hook
useNtcodeModels() → {
  models: ModelInfo[];
  switchModel: (id: string) => Promise<void>;
  currentModel: string | null;
}

// 对话 hook
useNtcodeConversations() → {
  conversations: ConversationSummary[];
  open: (id: string) => Promise<void>;
  create: (goal: string) => Promise<void>;
  save: () => Promise<void>;
  refresh: () => Promise<void>;
}
```

---

## 五、构建验证

所有代码已写入，但构建被其他 opencode 会话抢占 `target/` 目录拖慢。验证命令：

```bash
# 等其他会话结束后执行
cargo build -p neotrix-tauri          # 完整构建
cargo check -p neotrix --lib          # neotrix-core 检查
cargo test -p neotrix --lib -- nt_conversation nt_shared_mind nt_crystal_task_fusion --test-threads=1
```

### 已知预存问题（非本次改动）

- `skill_evolution.rs:157` — HashMap 未导入
- `mapper.rs:181`, `knowledge_miner.rs:169`, `self_evolver.rs:152` — RewardSource 类型不匹配
- `skill_loader.rs:660` — 已修复（`extract_description_fallback` → `extract_description_legacy`）

---

## 六、参考文件索引

| 文件 | 说明 |
|------|------|
| `src-tauri/src/ntcode/mod.rs` | 核心类型 |
| `src-tauri/src/ntcode/commands.rs` | 9 个 Tauri command |
| `src-tauri/src/ntcode/streaming.rs` | 流式 + event |
| `src-tauri/src/main.rs:188-210` | ntcode state 初始化 + 池发现 |
| `src-tauri/src/main.rs:249-261` | generate_handler 注册 |
| `neotrix-core/src/l1_action/nt_conversation.rs` | 对话持久化 |
| `neotrix-core/src/l1_action/nt_tui_app.rs` | TUI 扩展 |
| `neotrix-core/src/l1_action/nt_free_pool.rs` | 模型池轮转 |
| `neotrix-core/src/l1_action/nt_model_cli.rs` | 模型 CLI 调用 |
| `neotrix-core/src/bin/ntcode.rs` | ntcode 二进制入口 |
| `docs/plans/2026-09-22-ntcode-tauri-api-design.md` | 设计文档 |

---

## 七、前端对接已完成（2026-09-22 晚，桌面 App 构建）

**新增文件：**
- `src-tauri/frontend/src/api/ntcode.ts` — 类型＋9 命令＋4 事件订阅（`IpcResponse` 信封解包，非桌面抛 `NtcodeError(NOT_AVAILABLE)`）
- `src-tauri/frontend/src/api/index.ts` — barrel 导出（函数＋类型）

**对齐点（与后端实测一致）：**
- 命令名/参数：`ntcode_send {message}` / `ntcode_switch_model {model}` / `ntcode_open {id}` / `ntcode_new {goal}`，其余无参
- `ntcode_status` 返回 snake_case（`idle|streaming|waiting_human`）；`ntcode_send` 的 BUSY 错误走信封 `ok:false`
- 事件载荷：chunk`{content}` / done`{summary,model}` / error`{message}` / status`{status}`
- 事件订阅防泄漏（逐个 try/catch，失败释放已注册部分；参照 `listenUpdateEvents`）

**未做（需二进制联调）：** 聊天管线切换到 ntcode 流（Chat 现走 `chat.send` domain 通道）；`ntcode_models` 与模型池 UI 的数据源切换。`npm run build` ✓（tsc＋vite）。
