## Goal
- 极简桌面端（Tauri v2 + React/TypeScript），用户只管对话，意识核心自动运行

## Progress
### UI Design Refinements
- **CSS 重构**：2594→~470 行，删除所有旧组件样式（session/file-tree/split-view/virtual-os/agent-maker/agent-flow 等 40+ 节），自定义 CSS 变量 `--surface`/`--accent`/`--text-primary`/`--border` 双主题（light/dark）
- **呼吸灯面板**：EvolutionPanel 用 ⬡/⟁/⏣/◈ 象形文字 + `breathe` 动画，KnowledgePanel Awakening 用 Φ/FCS/USK 脉冲圈，纯状态显示无按钮
- **Store/Types 精简**：移除 EvolutionState/VirtualOS/AgentMaker/AgentFlow/FileTree 等废字段
- **SettingsModal 内联**到 App.tsx，删除 SettingsPanel.tsx
- **Sidebar 内联**到 App.tsx `nav`，删除 Sidebar.tsx

### Build Status
- `cargo check --lib -p neotrix`: 0 errors
- `cargo check -p neotrix-tauri`: 0 errors
- `npx tauri build --no-bundle`: ✅ (release binary)
- `npm run build` (frontend): ✅ (15KB CSS, 564KB JS)

## Relevant Files
- `src/styles/global.css`: 重构后设计系统
- `src/App.tsx`: Settings/Sidebar 内联
- `src/store.ts`: 精简
- `src/types.ts`: 精简
- `src/components/EvolutionPanel.tsx`: 呼吸灯
- `src/components/KnowledgePanel.tsx`: 呼吸灯 + Brain stats
