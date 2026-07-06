# preview-ui-v2.html → React 迁移计划

## 现有架构总览

```
React Frontend (src-tauri/frontend/src/)
├── stores/ (10 Zustand slices) ← 状态中心
├── lib/api.ts (80+ IPC wrappers) ← 后端通信
├── lib/persistence.ts (4 localStorage keys) ← 持久化
├── App.tsx (event listeners, shortcuts, theme) ← 根组件
├── components/ (50+ components) ← UI 组件
├── pages/ (11 route pages) ← 路由页面
└── router.tsx (14 routes) ← 路由配置

Rust Backend (src-tauri/src/)
├── commands/ (14 files, 80+ #[tauri::command]) ← IPC 命令
└── main.rs (command registration, event emit) ← 入口
```

## 通信链路标准模式

```
Component → useStore.action() → api.ts wrapper → invoke("command") → Rust
Rust → app.emit("event-name", payload) → App.tsx listen() → store action → component re-render
```

## 新功能实现清单（优先级排序）

### P0 — 当前架构已有的（直接复用）
- [x] SessionList (侧边栏会话)
- [x] ChatPanel (消息列表 + Markdown)
- [x] InputPanel (输入框)
- [x] StatusBar (底部状态栏)
- [x] 主题三态切换 (light/dark/system)
- [x] 快捷键系统 (Cmd+N/K/,/B/F + Esc)
- [x] SettingsPage + Settings (3-tab: Provider/通用/知识库)
- [x] CodeEditor + FileTree
- [x] ProxyPage (代理管理)
- [x] ConsciousnessBar (顶部意识栏)

### P1 — 需要从 HTML 迁移的 UI 功能
1. **UserPopover** — 用户弹出层（头像/名称/设置/主题/帮助/退出）
2. **Enhanced Settings** — 从 3-tab 扩展到 8-tab（+API/模型/隐私/快捷键/关于）
3. **MessageActionBar** — 消息悬停操作条（复制/重新生成/赞/踩）
4. **Attachments** — 附件选择 + Chip 显示 + 上传
5. **StreamingCursor** — 流式输出闪烁光标 + 思考中动画
6. **Pet** — 对齐 `ConsciousPet` 行为到 HTML 原型的动画规格

### P2 — 新视图
7. **CoworkView** — 团队协作视图（会话列表 + 任务看板 + Agent 状态）
8. **AgentDashboard** — 代理仪表板增强（Hero 环 + 谐振链可视化）

### P3 — 高级功能
9. **Cultivation/经脉** — 经脉修炼系统
10. **HelpOverlay** — 帮助弹窗

## 实施规范

### 新增 Store Slice 规范
```typescript
// stores/popoverSlice.ts
export interface PopoverSlice {
  popoverOpen: boolean;
  setPopoverOpen: (open: boolean) => void;
  togglePopover: () => void;
}
```

### 新增 api.ts 包装器规范
```typescript
// api.ts
export async function someCommand(param: string): Promise<Result> {
  return invoke<Result>("command_name", { param });
}
```

### 新增 Component 规范
```typescript
// components/SomeComponent.tsx
import { useStore } from "../stores";
import styles from "./SomeComponent.module.css";

const SomeComponent: React.FC<{ onAction: () => void }> = ({ onAction }) => {
  const value = useStore((s) => s.someValue);
  return <div className={styles.root}>{value}</div>;
};
export default SomeComponent;
```

### 路由注册规范
```typescript
// router.tsx — 新增子路由
{ path: "cowork", element: <CoworkPage /> },
```
