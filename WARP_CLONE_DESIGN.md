# Neotrix-Warp 克隆设计
> 基于 Neotrix 内核的 Warp 终端复刻设计
> 参考：Warp 架构分析报告（2026-04-29）

## 1. 设计目标
复刻 Warp 核心体验（Block 终端、AI 原生集成、现代 UI），底层完全复用 Neotrix 现有内核，利用 ReasoningBrain 自迭代、MCP 工具等能力，实现超越 Warp 的 AI 终端。

## 2. 架构映射
```mermaid
graph TB
    subgraph "Neotrix 内核层（复用）"
        RB[ReasoningBrain<br/>自迭代/SEAL/ReasoningBank]
        K[Kernel<br/>Ψ公式/沙箱/SCL]
        P[Parallel<br/>任务调度/多智能体]
        MCP[MCP Tools<br/>rmcp 0.5/Playwright/cua]
        WM[World Model<br/>RL 奖励]
        Sig[Signal<br/>信号系统]
    end

    subgraph "终端服务层（新增）"
        Term[终端模拟器<br/>Block 系统/输入编辑器]
        AI_Service[AI Agent 服务<br/>复用 RB]
        CLI_Int[CLI Agent 集成<br/>复用 MCP/Provider]
        WS[Workspace/Session<br/>管理]
    end

    subgraph "UI 层（新增）"
        Render[渲染引擎<br/>wgpu]
        UI_Frame[UI 框架<br/>Entity-Component]
        Scene_G[Scene Graph<br/>Layer/Rect/Image/Glyph]
        Event[事件系统<br/>发布-订阅]
    end

    内核层 --> 终端服务层
    终端服务层 --> UI 层
```

## 3. 核心模块设计

### 3.1 渲染引擎（替代 Warp 的 Metal/OpenGL）
- **技术选型**：wgpu（Rust 跨平台 GPU 库）
- **优势**：
  - 一次编写，支持 Metal (macOS)、Vulkan (Linux/Windows)、WebGPU (Web)
  - 与 Neotrix Rust 技术栈统一，Warp 需分别实现 3 套渲染后端
- **实现参考**：Warp 的 Scene Graph 设计，仅渲染 Rect/Image/Glyph 三种图元

### 3.2 UI 框架（替代 WarpUI）
- **模式**：参考 Warp 的 Entity-Component-Handle，结合 Neotrix 类型系统强化安全
- **核心类型**：
  ```rust
  // 复用 Neotrix 原子自增 ID 模式（reasoning_brain/core.rs）
  pub struct EntityId(usize);
  impl EntityId {
      pub fn new() -> Self {
          static NEXT: AtomicUsize = AtomicUsize::new(0);
          Self(NEXT.fetch_add(1, Ordering::Relaxed))
      }
  }

  // ViewHandle 结合 Neotrix Signal 系统
  pub struct ViewHandle<T> {
      entity_id: EntityId,
      ref_counts: Weak<Mutex<RefCounts>>,
      _marker: PhantomData<T>,
  }
  ```

### 3.3 终端模拟器（复刻 Warp 核心体验）
- **Block 系统**：
  - 每个 Block 对应 Neotrix Signal 单元，包含 `Input`（命令）+ `Output`（输出）
  - 复用 ReasoningBank 存储 Block 历史，替代 Warp Drive
  - 命令执行走 Kernel 沙箱，更安全
- **输入编辑器**：
  - 复用 Kernel 的 SCL 语言支持，实现多行编辑、语法高亮
  - 结合 Provider 模块实现多模型命令补全

### 3.4 AI 集成（超越 Warp）
- **内置 Agent**：直接复用 ReasoningBrain，替代 Warp Oz：
  - 支持 SEAL 自迭代、能力向量动态调整、ReasoningBank 经验存储
  - 多模型支持（OpenAI/Anthropic/Gemini/Ollama），Warp Oz 仅支持 GPT
- **第三方 CLI Agent**：复用 MCP Tools + Provider，支持 Claude Code/Codex/Gemini CLI：
  - 无需单独写插件，通过 MCP 协议直接调用
  - 支持自定义 Agent，走 Kernel 执行

## 4. 技术选型对比
| 模块 | Warp 实现 | 本设计实现 | 优势 |
|------|-----------|------------|------|
| 渲染引擎 | Metal/OpenGL/WebGL 分端实现 | wgpu 统一实现 | 跨平台，维护成本低 |
| AI 内核 | Oz（GPT 驱动，功能单一） | Neotrix ReasoningBrain | 自迭代、多模型、SEAL 循环 |
| 工具集成 | 自定义插件系统 | Neotrix MCP Tools | 开箱即用，支持更多工具 |
| 存储 | Diesel+SQLite + Warp Drive | ReasoningBank + Neotrix 持久化 | 经验存储，支持自学习 |
| 终端执行 | NuShell/Alacritty | Neotrix Kernel 沙箱 | 更安全，支持 SCL 语言 |

## 5. 实施步骤（参考 AGENTS.md 工作流）

### 阶段 1：基础设施
1. 新增 `src/neotrix/terminal/` 模块，参考 Warp `app/src/terminal/` 结构
2. 集成 wgpu，实现基础渲染管线（Rect/Image/Glyph）
3. 实现 Entity-Component-Handle 模式，复用 Neotrix `EntityId` 逻辑

### 阶段 2：终端模拟器
1. 实现 ANSI 转义序列解析，兼容现有终端应用
2. 实现 Block 系统，每个 Block 关联 Neotrix Signal
3. 实现输入编辑器，支持多行编辑、自动补全

### 阶段 3：AI 集成
1. 对接 ReasoningBrain 到终端 UI，实现内置 Agent
2. 对接 MCP Tools，支持第三方 CLI Agent 调用
3. 实现 Workspace/Session 管理，复用 ReasoningBank 存储

### 阶段 4：优化
1. 跨平台支持（Linux/Windows）
2. 实现 Warp 特色功能（Block 分享、AI 建议）
3. 确保 `cargo check --lib` 零错误零警告

## 6. 关键设计决策
1. **不采用 WarpUI 代码**：WarpUI 为 AGPL v3 许可证，避免许可证冲突，参考设计自研
2. **用 wgpu 替代多后端渲染**：Neotrix 是 Rust 项目，wgpu 原生适配，减少维护成本
3. **直接复用 ReasoningBrain**：已具备 Oz 所有能力且更强，无需重复实现
4. **Block 走 Signal 系统**：统一 Neotrix 信号模型，便于自迭代优化
