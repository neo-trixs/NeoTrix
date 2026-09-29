# ntcode Tauri API 设计

> 将 ntcode 构建为桌面 app 的默认对话模型，前端围绕对话界面进行可视化。

## 架构

```
前端 (React/Vue 对话界面)
  │ invoke("ntcode_send", { message })
  │ listen("ntcode://chunk", callback)
  ▼
Tauri IPC (commands/ntcode.rs)
  │ NtcodeState (Arc<RwLock<NtcodeSession>>)
  ▼
neotrix-core
  │ NtInnerLoop::drive_with_sink()
  │ NtFreePoolAsk (模型池轮转)
  │ CrystalCore (晶体记忆)
  ▼
opencode CLI (外部模型调用)
```

## 核心类型

### NtcodeSession（会话状态）

```rust
pub struct NtcodeSession {
    pub conversation_id: Option<String>,
    pub goal: String,
    pub model: Option<String>,        // 当前定点模型
    pub transcript: Vec<ChatMessage>, // 对话历史
    pub status: SessionStatus,
}

pub struct ChatMessage {
    pub role: Role,        // User | Assistant | System
    pub content: String,
    pub timestamp: u64,
    pub model: Option<String>,
}

pub enum SessionStatus {
    Idle,
    Streaming { started_at: Instant },
    WaitingHuman,
}
```

### NtcodeState（Tauri managed state）

```rust
pub struct NtcodeState {
    inner: Arc<RwLock<NtcodeSession>>,
    pool: Arc<NtFreePoolAsk>,
    pool_lines: Vec<String>,
}
```

## Tauri Commands

| Command | 输入 | 输出 | 说明 |
|---------|------|------|------|
| `ntcode_send` | `{ message: String }` | `IpcResponse<()>` | 发送消息，流式响应走 event |
| `ntcode_stop` | — | `IpcResponse<()>` | 停止当前生成 |
| `ntcode_models` | — | `IpcResponse<Vec<ModelInfo>>` | 获取可用模型列表 |
| `ntcode_switch_model` | `{ model: String }` | `IpcResponse<()>` | 切换模型 |
| `ntcode_conversations` | — | `IpcResponse<Vec<ConversationSummary>>` | 对话列表 |
| `ntcode_open` | `{ id: String }` | `IpcResponse<()>` | 打开对话 |
| `ntcode_new` | `{ goal: String }` | `IpcResponse<()>` | 新建对话 |
| `ntcode_save` | — | `IpcResponse<()>` | 保存当前对话 |
| `ntcode_status` | — | `IpcResponse<SessionStatus>` | 当前状态 |

## Tauri Events（流式推送）

| Event | Payload | 说明 |
|-------|---------|------|
| `ntcode://chunk` | `{ content: String }` | LLM 流式文本片段 |
| `ntcode://done` | `{ summary: String, status: String }` | 生成完成 |
| `ntcode://error` | `{ message: String }` | 错误 |
| `ntcode://demands` | `{ demands: Vec<DemandInfo> }` | 内需上窗 |
| `ntcode://status` | `{ status: String }` | 状态变更 |

## 前端组件结构

```
App
├── ConversationHeader     # 对话标题 + 模型选择器
├── MessageList            # 消息列表（流式渲染）
│   ├── MessageBubble      # 单条消息（User/Assistant/System）
│   └── StreamingIndicator # 生成中指示器
├── InputBar               # 输入框 + 发送按钮
├── Sidebar                # 对话列表 + 设置
│   ├── ConversationList   # 历史对话
│   └── ModelPicker        # 模型切换
└── StatusBar              # 状态栏（模型/状态/轮次）
```

## 实现步骤

1. `src-tauri/src/ntcode/mod.rs` — NtcodeSession, ChatMessage, SessionStatus
2. `src-tauri/src/ntcode/commands.rs` — 9 个 Tauri command
3. `src-tauri/src/ntcode/streaming.rs` — 后台流式任务 + event 推送
4. `src-tauri/src/lib.rs` — 注册 module + commands + state
5. `src-tauri/src/main.rs` — generate_handler! 添加 ntcode commands
