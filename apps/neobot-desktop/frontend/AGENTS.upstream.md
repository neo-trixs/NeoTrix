<!--
  ⛔⛔ 这是 **vendored 上游的** AGENTS.md（dsh-harness-desktop 0.19.1，MIT），
  **不是本仓的开发规范**。原名是 AGENTS.md，1:1 复制时被带进来。

  为什么必须改名：
    它以「项目规范」的身份出现在**编辑路径**里，编辑本仓文件时会被当作约束读入。
    而它要求「0 注释原则」「严禁查看 archive/」—— 那是上游对自己仓的要求，
    与本仓的既定做法直接冲突：本仓把**为什么**记在代码里
    （STATUS.md §4 记了 7 次假绿，全靠这些注释撑着）。

  ⛔ **vendored 项目的规则不能覆盖宿主项目的约定。**
    一条写错的、被当成规范执行的指令，比没有指令更危险 —— 它会让人
    主动删掉正确的东西，并且看起来是在「遵守规范」。

  本仓规范见：仓库根 AGENTS.md、RUST-STANDARDS.md、apps/neobot-desktop/STATUS.md。
-->
## Workspace Routing & Context Guidelines

在执行代码分析、优化、重构或单测编写前，必须根据修改对象的**路径、业务域与环境**，主动读取并叠加遵循对应的规范文档：

### 1. 规范矩阵与路由表 (Specification Routing Matrix)

| 修改范围 / 触发条件 | 业务场景 / 对应模块 | 必须参照的规范文档 |
| :--- | :--- | :--- |
| **基础通用** | 所有代码修改 | `docs/specs/devlopment.md` |
| **测试标准** | 通用 Unit Test、E2E 集成测试断言与结构 | `docs/specs/testing.md` |
| **桌面端 - 基础架构** | 桌面端主进程、Shell 基础、GUI 渲染核心 | `docs/specs/desktop.baisc.md` |
| **桌面端 - 测试验证** | 桌面端 E2E 测试、UI 单元测试、桌面功能测试 | `docs/specs/desktop.test.md` |
| **插件 - 基础架构** | 插件常量归属、跨侧共享约定与退级策略 | `docs/specs/plugin.baisc.md` |
| **插件 - 客户端核心** | 插件 Client 通信、状态管理与客户端逻辑 | `docs/specs/plugin.client.md` |
| **插件 - 客户端面板** | 插件 Panel 视图组件、交互面板 UI | `docs/specs/plugin.client.panel.md` |
| **插件 - 宿主核心** | 宿主进程管理、插件加载器、沙箱隔离 | `docs/specs/plugin.host.md` |
| **插件 - 宿主服务** | 宿主系统级 Service、底座 API 实现 | `docs/specs/plugin.host.service.md` |
| **插件 - 测试套件** | 插件 Mock 测试、沙箱运行测试、集成测试 | `docs/specs/plugin.test.md` |
| **上游同步/依赖管理** | Upstream 仓库同步、依赖重构、版本兼容 | `docs/specs/upstram.sync.md` |

### 2. 规范叠加与优先级 (Specification Stacking Rules)
- **通用基础**：`docs/specs/devlopment.md` 为全局必须叠加的基础规范。
- **多模块协同**：若同时跨越桌面端与插件（例如：优化插件面板及其与桌面宿主的通信），**必须同时读取并整合**涉及的所有子规范。
- **冲突裁决**：当规则冲突时，优先级为 `专项/业务规范 > 运行环境规范 > devlopment.md`。

---

## 最高优先级 (Strict Guidelines)

1. **隐私与目录隔离**：
   - 严禁查看 `archive/` 目录下的任何内容，除非用户明确提及并给予特殊许可。
2. **极致代码精简与注释策略**：
   - **0 注释原则**：编写/重构代码时必须保持 **0 注释**，仅允许在极难通过代码自解释的**关键节点/复杂算法**添加微量注释。
   - **清理冗余**：优化时若发现代码中存在大面积文档描述、历史残余注释或冗余代码，应主动清理/精简，提升干净度。
3. **临时草稿区**：
   - 若需要临时记录、存放中间分析过程或草稿，可随意在 `.temp` 文件中编写（该文件下拥有绝对自由度）。
4. **插件开发保护**：
   - 在处理或重构插件相关代码时，**切勿执行构建（Build）命令**（用户可能正处于 Dev 开发热重载状态）。

---

## Command Performance Constraints (Strictly Enforced)

执行文件查找或代码检索时，必须遵守极速命令约束，避免触发低效遍历：

1. **FORBIDDEN POWERSHELL COMMANDS:**
   - **NEVER** use `Get-ChildItem -Recurse` or `dir -s` to search files/directories.
   - **NEVER** use `Select-String -Path` for recursive text searches.

2. **FAST ALTERNATIVES (MANDATORY):**
   - **Searching text in files:** Use `rg "pattern"` (ripgrep).
   - **Finding files by name/path:** Use `fd <pattern>` or `git ls-files | Select-String "pattern"`.
   - **Listing top-level directory items:** Use simple `Get-ChildItem` (NO `-Recurse`).

3. **GIT PROJECT EXEMPTION:**
   - Always leverage Git index if available: `git ls-files` is exponentially faster than PowerShell directory traversal.