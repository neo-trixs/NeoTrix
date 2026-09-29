# TUI 对话管理 + 模型切换 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 TUI 支持：切换模型、新增对话、切换对话、列出对话历史。

**Architecture:** 新增 `nt_conversation.rs` 模块管理对话持久化（JSON 文件 per 对话）。TUI app 扩展状态管理当前对话 ID + 对话列表。新增 slash 命令 + 键盘快捷键。

**Tech Stack:** `serde`/`serde_json`（已有）、`std::fs`（原子写）、`chrono`（时间戳）、`crossterm`（键盘事件）。

---

### Task 1: 对话持久化模块

**Files:**
- Create: `neotrix-core/src/l1_action/nt_conversation.rs`
- Modify: `neotrix-core/src/l1_action/mod.rs`

**Step 1: 定义 Conversation 结构体**

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub goal: String,
    pub transcript: Vec<String>,
    pub model: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Conversation {
    pub fn new(goal: &str, model: Option<String>) -> Self {
        let now = unix_now();
        Self {
            id: format!("conv-{:x}", now),
            goal: goal.to_string(),
            transcript: Vec::new(),
            model,
            created_at: now,
            updated_at: now,
        }
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 对话目录：~/.neotrix/conversations/
fn conv_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("conversations")
}

/// 保存对话到 JSON 文件
pub fn save(conversation: &Conversation) -> Result<(), String> {
    let dir = conv_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建对话目录失败: {e}"))?;
    let path = dir.join(format!("{}.json", conversation.id));
    let json = serde_json::to_string_pretty(conversation)
        .map_err(|e| format!("序列化失败: {e}"))?;
    // 原子写：写 tmp 再 rename
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json).map_err(|e| format!("写入失败: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("重命名失败: {e}"))?;
    Ok(())
}

/// 加载指定对话
pub fn load(id: &str) -> Result<Conversation, String> {
    let path = conv_dir().join(format!("{id}.json"));
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取对话 {id} 失败: {e}"))?;
    serde_json::from_str(&json)
        .map_err(|e| format!("解析对话 {id} 失败: {e}"))
}

/// 列出所有对话（按 updated_at 降序）
pub fn list() -> Vec<Conversation> {
    let dir = conv_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut convs: Vec<Conversation> = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .filter_map(|e| {
            let json = std::fs::read_to_string(e.path()).ok()?;
            serde_json::from_str(&json).ok()
        })
        .collect();
    convs.sort_by(|a, b| b.updated_at.cached_cmp(&a.updated_at));
    convs
}

/// 删除对话
pub fn delete(id: &str) -> Result<(), String> {
    let path = conv_dir().join(format!("{id}.json"));
    std::fs::remove_file(&path).map_err(|e| format!("删除对话 {id} 失败: {e}"))
}
```

**Step 2: 在 mod.rs 中导出**

在 `neotrix-core/src/l1_action/mod.rs` 中添加：
```rust
pub mod nt_conversation;
```

**Step 3: 验证编译**

Run: `cargo check -p neotrix --lib`
Expected: clean

**Step 4: Commit**

```bash
git add neotrix-core/src/l1_action/nt_conversation.rs neotrix-core/src/l1_action/mod.rs
git commit -m "feat(nt_conversation): dialogue persistence module"
```

---

### Task 2: TUI 状态扩展 — 对话管理字段

**Files:**
- Modify: `neotrix-core/src/l1_action/nt_tui_app.rs`

**Step 1: NtTuiApp 新增字段**

```rust
pub struct NtTuiApp {
    state: NtTuiState,
    pool: Arc<NtFreePoolAsk>,
    pool_lines: Vec<String>,
    active_tasks: Vec<WorkingView>,
    quit: bool,
    // --- new ---
    current_conv_id: Option<String>,  // 当前对话 ID（None = 未保存的新对话）
    conversations: Vec<ConversationSummary>, // 对话列表摘要
}

struct ConversationSummary {
    id: String,
    goal: String,
    updated_at: u64,
    model: Option<String>,
}
```

**Step 2: 修改 new() 方法签名**

在 `NtTuiApp::new()` 中增加参数 `conv_id: Option<String>`，初始化新字段。

**Step 3: 添加 save/load 方法**

```rust
impl NtTuiApp {
    /// 保存当前对话到磁盘
    fn save_current(&mut self) {
        let conv = Conversation {
            id: self.current_conv_id.clone()
                .unwrap_or_else(|| format!("conv-{:x}", unix_now())),
            goal: self.state.goal.clone(),
            transcript: self.state.transcript.clone(),
            model: self.pool.pinned(),
            created_at: 0, // preserve existing
            updated_at: unix_now(),
        };
        if nt_conversation::save(&conv).is_ok() {
            self.current_conv_id = Some(conv.id);
        }
    }

    /// 加载对话列表
    fn refresh_conversations(&mut self) {
        self.conversations = nt_conversation::list()
            .into_iter()
            .map(|c| ConversationSummary {
                id: c.id,
                goal: c.goal,
                updated_at: c.updated_at,
                model: c.model,
            })
            .collect();
    }

    /// 打开指定对话
    fn open_conversation(&mut self, id: &str) -> Result<(), String> {
        let conv = nt_conversation::load(id)?;
        self.state.goal = conv.goal;
        self.state.transcript = conv.transcript;
        self.current_conv_id = Some(conv.id);
        // 恢复模型
        if let Some(model) = conv.model {
            self.pool.set_pinned(Some(model));
        }
        Ok(())
    }

    /// 新建对话（清空当前状态）
    fn new_conversation(&mut self, goal: &str) {
        // 先保存当前对话
        if self.current_conv_id.is_some() {
            self.save_current();
        }
        self.state = NtTuiState::new(
            goal,
            &String::new(),
            &self.state.pool_line,
            0,
            Vec::new(),
        ).with_pool_models(self.pool_models());
        self.current_conv_id = None;
        self.active_tasks.clear();
    }
}
```

**Step 4: 验证编译**

Run: `cargo check -p neotrix --lib`
Expected: clean (may have unused warnings, fine)

**Step 5: Commit**

```bash
git add neotrix-core/src/l1_action/nt_tui_app.rs
git commit -m "feat(nt_tui_app): conversation management fields and methods"
```

---

### Task 3: Slash 命令扩展

**Files:**
- Modify: `neotrix-core/src/l1_action/nt_tui_app.rs`

**Step 1: NtSlash 枚举扩展**

```rust
pub enum NtSlash {
    Help,
    Pool,
    Model,
    Quit,
    Clear,
    // --- new ---
    New,              // /new [goal] — 新建对话
    List,             // /list — 列出对话
    Open(String),     // /open <id> — 打开对话
    Save,             // /save — 保存当前对话
    Unknown(String),
}
```

**Step 2: dispatch_slash 解析扩展**

```rust
pub fn dispatch_slash(line: &str) -> NtSlash {
    let trimmed = line.trim();
    if !trimmed.starts_with('/') {
        return NtSlash::Unknown(trimmed.to_string());
    }
    let parts: Vec<&str> = trimmed[1..].splitn(2, |c: char| c.is_whitespace()).collect();
    match parts[0] {
        "help" | "h" => NtSlash::Help,
        "pool" | "p" => NtSlash::Pool,
        "model" | "m" => NtSlash::Model,
        "quit" | "exit" | "q" => NtSlash::Quit,
        "clear" | "cls" => NtSlash::Clear,
        "new" | "n" => NtSlash::New(parts.get(1).map(|s| s.to_string()).unwrap_or_default()),
        "list" | "ls" => NtSlash::List,
        "open" | "o" => NtSlash::Open(parts.get(1).unwrap_or(&"").to_string()),
        "save" | "s" => NtSlash::Save,
        other => NtSlash::Unknown(other.to_string()),
    }
}
```

**Step 3: app_slash 处理扩展**

在 `app_slash()` 方法中添加 match 分支：

```rust
NtSlash::New(goal) => {
    app.save_current();
    app.new_conversation(if goal.is_empty() { "新对话" } else { &goal });
    app.state.transcript.push("新对话已创建。".into());
}
NtSlash::List => {
    app.refresh_conversations();
    if app.conversations.is_empty() {
        app.state.transcript.push("暂无保存的对话。".into());
    } else {
        app.state.transcript.push("══ 对话列表 ══".into());
        for c in &app.conversations {
            let model = c.model.as_deref().unwrap_or("轮转");
            let ts = format_timestamp(c.updated_at);
            let marker = if app.current_conv_id.as_deref() == Some(&c.id) { " ◀" } else { "" };
            app.state.transcript.push(
                format!("  [{}] {} ({}) — {}{marker}", &c.id[..12], c.goal, model, ts)
            );
        }
    }
}
NtSlash::Open(id) => {
    if id.is_empty() {
        app.state.transcript.push("用法：/open <对话ID>".into());
    } else {
        match app.open_conversation(&id) {
            Ok(()) => {
                app.state.transcript.push(format!("已打开对话 {id}"));
            }
            Err(e) => {
                app.state.transcript.push(format!("打开失败：{e}"));
            }
        }
    }
}
NtSlash::Save => {
    app.save_current();
    app.state.transcript.push("对话已保存。".into());
}
```

**Step 4: 验证编译**

Run: `cargo check -p neotrix --lib`
Expected: clean

**Step 5: Commit**

```bash
git add neotrix-core/src/l1_action/nt_tui_app.rs
git commit -m "feat(nt_tui_app): /new /list /open /save slash commands"
```

---

### Task 4: 键盘快捷键

**Files:**
- Modify: `neotrix-core/src/l1_action/nt_tui_app.rs`

在 `app_key()` 函数中添加：

```rust
// Ctrl+N: 新建对话
KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
    app.new_conversation("新对话");
    app.state.transcript.push("新对话已创建。".into());
}
// Ctrl+L: 列出对话
KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
    app.refresh_conversations();
    // 渲染到 transcript（复用 /list 逻辑）
    if app.conversations.is_empty() {
        app.state.transcript.push("暂无保存的对话。".into());
    } else {
        app.state.transcript.push("══ 对话列表 ══".into());
        for c in &app.conversations {
            let model = c.model.as_deref().unwrap_or("轮转");
            let ts = format_timestamp(c.updated_at);
            app.state.transcript.push(
                format!("  [{}] {} ({})", &c.id[..12], c.goal, model)
            );
        }
    }
}
// Ctrl+O: 打开对话（弹出选择器）— 暂用简单列表，后续可做 picker
KeyCode::Char('o') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
    // 打开最近的对话（或提示 /open）
    app.state.transcript.push("输入 /open <对话ID> 打开对话".into());
}
// Ctrl+S: 保存对话
KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
    app.save_current();
    app.state.transcript.push("对话已保存。".into());
}
```

**Step 2: 验证编译**

Run: `cargo check -p neotrix --lib`

**Step 3: Commit**

```bash
git add neotrix-core/src/l1_action/nt_tui_app.rs
git commit -m "feat(nt_tui_app): Ctrl+N/L/O/S keyboard shortcuts"
```

---

### Task 5: 帮助面板更新

**Files:**
- Modify: `neotrix-core/src/l1_action/nt_tui_app.rs`

在 `render_help_app()` 中添加新快捷键说明：

```rust
// 在现有帮助文本后追加：
"  Ctrl+N        新建对话\n\
   Ctrl+L        对话列表\n\
   Ctrl+S        保存对话\n\
   /new [目标]   新建对话\n\
   /list         对话列表\n\
   /open <ID>    打开对话\n\
   /save         保存对话\n\
   /model        选择模型\n"
```

**Step 2: 验证编译**

Run: `cargo check -p neotrix --lib`

**Step 3: Commit**

```bash
git add neotrix-core/src/l1_action/nt_tui_app.rs
git commit -m "feat(nt_tui_app): update help panel with conversation shortcuts"
```

---

### Task 6: 状态栏增强

**Files:**
- Modify: `neotrix-core/src/l1_action/nt_tui_app.rs`

在 `render_status_app()` 中显示当前对话 ID 和模型：

```rust
// 状态栏右侧追加：
let conv_tag = app.current_conv_id
    .as_ref()
    .map(|id| format!("对话:{}", &id[..8]))
    .unwrap_or_else(|| "新对话".into());
let model_tag = app.pool.pinned()
    .unwrap_or_else(|| "轮转".into());
// 合并到状态栏
```

**Step 2: 验证编译 + 运行测试**

Run: `cargo check -p neotrix --lib && cargo test -p neotrix --lib -- nt_tui_app --test-threads=1`

**Step 3: Commit**

```bash
git add neotrix-core/src/l1_action/nt_tui_app.rs
git commit -m "feat(nt_tui_app): status bar shows current conversation and model"
```

---

### Task 7: ntcode 二进制入口适配

**Files:**
- Modify: `neotrix-core/src/bin/ntcode.rs`

在 TUI 模式启动时传入 `conv_id: None`（新对话），确保 `run_tui_session` 签名兼容。

**Step 2: 验证构建**

Run: `cargo build -p neotrix --bin ntcode`

**Step 3: Commit**

```bash
git add neotrix-core/src/bin/ntcode.rs
git commit -m "feat(ntcode): wire conversation management to TUI entry"
```

---

### Task 8: 端到端验证

**Step 1: 构建 + 测试**

```bash
cargo build -p neotrix --bin ntcode
cargo test -p neotrix --lib -- nt_tui_app nt_conversation --test-threads=1
```

**Step 2: TUI 手动验证**

```bash
./target/debug/ntcode "测试对话" --line --model opencode/mimo-v2.6-flash-free
```

测试：
- `/model` 打开 picker
- `/list` 显示空列表
- `/save` 保存对话
- `/new 新目标` 创建新对话
- `/list` 显示 2 个对话
- `/open conv-xxx` 打开旧对话

**Step 3: 最终 Commit**

```bash
git add -A
git commit -m "feat(ntcode): complete conversation management and model switching"
```
