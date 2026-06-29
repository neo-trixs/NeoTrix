# NeoTrix Desktop

## 🚀 项目概述

NeoTrix Desktop 是一个基于 **NeoTrix 原理体系** 构建的 **极简、高效、模块化** 的桌面应用框架。

### 核心技术理念
- **Thin Shell + Rich Core** - 精致的窗口界面，强大的内部逻辑
- **Zero Dependency** - 自包含的开发工具，易于部署
- **Hot Reloadable** - 支持实时更新，开发体验极佳
- **Component Pattern** - 独立、可插拔、可测试的组件
- **Progressive Disclosure** - 按需展示功能，降低认知负荷

### 目标

- 构建一个 **高颜值、好体验、易扩展** 的桌面应用框架
- 提供 **专业、高效、易用** 的 AI 智能助手功能
- 实现 **跨平台** (Windows, macOS, Linux) 支持
- 支持 **多语言** (中英文)  localization
- 提供 **开箱即用** 的开发体验

---

## 📋 快速开始

### 1. 安装依赖

```bash
# 克隆项目
git clone <repo>
cd neotrix-desktop

# 安装依赖 (Windows / macOS / Linux 通用)
npm install

# 或一键设置 (自动 install + build)
npm run setup
```

### 2. 开发模式

```bash
# 启动开发服务器 (热重载)
npm run dev

# 打开 http://localhost:5173

# 构建生产版本
npm run build

# 启动预览服务器
npm run preview
```

### 3. 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `⌘B` / `^B` | 切换侧边栏 |
| `⌘K` / `^K` | 搜索聚焦 |
| `Esc` | 失焦 |

### 4. 运行测试

```bash
# 运行单元测试
pnpm test

# 运行集成测试
pnpm test:integration

# 代码质量检查
pnpm lint

# TypeScript 类型检查
pnpm typecheck
```

---

## 📁 项目结构

```
neotrix-desktop/
├── public/
│   ├── index.html        # HTML 入口 (zh-CN, PWA meta)
│   ├── favicon.svg        # SVG favicon
│   └── manifest.json      # PWA manifest
├── src/
│   ├── components/        # UI 组件
│   │   ├── TopNavBar.tsx  # 顶部导航栏 (搜索/主题/布局切换)
│   │   ├── Sidebar.tsx    # 侧边栏 (导航 + 快捷操作)
│   │   ├── CenterPanel.tsx # 主内容区容器
│   │   ├── RightPanel.tsx  # 右侧面板 (工具/插件/洞察)
│   │   └── StatusBar.tsx   # 底部状态栏
│   ├── core/              # 核心模块
│   │   ├── experience-tree.ts # 经验树 (衰减/强化/修剪)
│   │   └── todo-system.ts     # 任务系统 (CRUD + 统计)
│   ├── hooks/             # React Hooks
│   │   ├── useApp.ts      # 全局状态 (Context + Reducer)
│   │   ├── useKeyboard.ts # 键盘快捷键
│   │   ├── useMediaQuery.ts # 响应式查询
│   │   └── useSystemTheme.ts # 系统主题检测
│   ├── styles/
│   │   ├── design-tokens.scss   # 80+ CSS 变量 (暗/亮主题)
│   │   ├── typography.scss     # 中英文排版优化
│   │   ├── layout.scss         # 三面板布局
│   │   ├── components.scss     # 所有组件样式
│   │   └── interactions.scss   # 交互/滚动条/动画
│   ├── App.tsx             # 主应用 (三面板 + 路由)
│   ├── App.css             # 全局重置 + 路由页面样式
│   └── index.tsx           # 入口 (QueryClient + Provider + Router)
├── src-tauri/             # Tauri 原生壳 (可选)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/main.rs
├── scripts/
│   ├── setup.js           # 跨平台安装脚本
│   └── dev.js             # 跨平台开发启动器
├── .editorconfig            # 编辑器配置
├── .gitignore
├── .eslintrc.cjs            # ESLint 配置
├── .prettierrc              # Prettier 配置
├── package.json
├── tsconfig.json
├── vite.config.js
└── README.md
```

---

## 🏗️ 技术栈

### 前端技术

| 技术 | 版本 | 用途 |
|------|------|-----|
| **React** | 18.2+ | UI 框架 |
| **TypeScript** | 5.0+ | 类型安全 (strict mode) |
| **Vite** | 5.4+ | 开发服务器和构建 |
| **SCSS** | 1.63+ | CSS 预处理器 |
| **CSS Variables** | — | 设计令牌 + 运行时主题切换 |

### 状态管理

| 技术 | 用途 |
|------|------|
| **React Context + useReducer** | 全局状态 (sidebar/theme/layout/session) |
| **React Router v6** | 客户端路由 |

### 原生壳 (可选)

| 技术 | 用途 |
|------|------|
| **Tauri v2** | 跨平台桌面壳 (macOS/Windows/Linux) |

---

## 🚀 开发指南

### 1. 项目启动

```bash
cd neotrix-desktop
npm install
npm run dev     # http://localhost:5173
```

### 2. 开发常用命令

| 命令 | 说明 |
|------|------|
| `npm run dev` | 启动开发服务器 (热重载) |
| `npm run build` | TypeScript 检查 + 生产构建 |
| `npm run preview` | 预览生产版本 |
| `npm run lint` | ESLint 检查 |
| `npm run typecheck` | TypeScript 类型检查 |
| `npm run setup` | 一键安装 + 构建 |
| `npm run clean` | 清理 dist 和 node_modules |

### 3. 项目结构约定

#### File Naming Conventions
- **Components**: `PascalCase` (e.g., `SideBar.tsx`)
- **Hooks**: `kebab-case` (e.g., `use-app.ts`)
- **Utils**: `kebab-case` (e.g., `date-utils.ts`)
- **Types**: `kebab-case` (e.g., `app-types.ts`)

#### Directory Organization
- `src/components/` - UI components
- `src/core/` - Core modules
- `src/ui/` - UI routing
- `src/hooks/` - React hooks
- `src/utils/` - Utility functions
- `src/types/` - TypeScript definitions
- `src/styles/` - Style resources
- `src/plugins/` - Plugin system

### 4. 代码风格指南

#### Code Style
- **缩进风格**: Use `prettier` to format code
- **ESLint**: Use `ESLint` for code checking
- **TypeScript**: Use `strict mode`
- **Naming Convention**: Follow MIPS naming convention

#### Documentation Style
- **注释**: Use JSDoc annotations
- **Documentation**: Use Markdown format
- **README**: Include project description, setup instructions, contribution guide

---

## 📦 包依赖

```json
{
  "name": "neotrix-desktop",
  "version": "0.1.0",
  "private": true,
  "description": "NeoTrix Desktop - 基于原理图设计的极简高效桌面应用",
  "author": "NeoTrix Team",
  "license": "MIT",
  "keywords": [
    "neotrix",
    "desktop",
    "react",
    "typescript",
    "ui",
    "design",
    "components"
  ],
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext .js,.jsx,.ts,.tsx",
    "test": "jest",
    "test:integration": "jest --config jest.integration.config.js",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.8.0",
    "react-query": "^4.8.0",
    "react-hot-toast": "^2.1.0"
  },
  "devDependencies": {
    "vite": "^4.10.0",
    "typescript": "^5.0.0",
    "eslint": "^8.34.0",
    "prettier": "^2.8.3",
    "jest": "^29.5.0",
    "@types/node": "^18.0.0",
    "@testing-library/react": "^13.4.1"
  }
}
```

---

## 🏆 项目目标

### Core Goals

1. **极简性**: Provide a zero burden development experience, focusing on core functionality
2. **高质量**: Through automated testing and code inspection, ensure code quality
3. **可扩展性**: Plugin architecture, supports third-party extensions
4. **文化适应性**: Support中文英文,适应不同文化背景的用户
5. **高性能**: Optimize rendering and interaction experience, provide smooth user experience

### Vision

Through **NeoTrix Desktop**, we hope to:
- Provide users with a **professional, efficient, easy-to-use** AI assistant
- Build an **open-source, extendable** desktop application development platform
- Create a **collaborative, innovative** developer community

---

## 🎯 最后祝愿

Wishing NeoTrix Desktop to be your **NeoTrix ecosystem** **core desktop component**, providing **high-quality user experience** for users, while offering **perfect development platform** for developers.

---

*Document Version: 0.1.0*
*Last Updated: 2024-06-25*
*Status: Development*

---

> This is a continuously evolving project that will be updated based on actual development needs and user feedback. Welcome to join our community and co-create the NeoTrix ecosystem!

---

## 🌟 Acknowledgment

Thanks to all developers who have contributed to NeoTrix Desktop.

For any questions, suggestions, or contributions, please visit:\n- **GitHub**: https://github.com/username/neotrix-desktop\n- **Issues**: https://github.com/username/neotrix-desktop/issues\n- **Discussions**: https://github.com/username/neotrix-desktop/discussions\n
Let us make NeoTrix Desktop better together!

---
\n🎯 **上一步任务**\n\n**Phase 1 - 核心基础组件开发 (下周优先级)**\n\n1. **完成顶部导航栏组件** - 品牌标识、导航、主题切换、布局
2. **完成应用状态管理 Hook** - Context Provider、React Query、Hook 导出
3. **完成路由系统集成** - 路由配置、保护、状态持久化\n\n**Phase 2 - 侧边栏面板开发 (下下周)**\n1. **侧边栏组件开发** - 会话管理、导航\n2. **右侧面板组件开发** - 工具管理\n3. **基础聊天界面开发** - 会话显示、输入功能\n\n---\n\n**📋 当前状态**: 🎯 **设计阶段** → 🚀 **实施阶段** → 📅 **成熟阶段**\n**预计完成**: 📅 7月下旬\n\n如果您有任何修改或需要，我将随时准备调整项目方案以满足项目需求！\n\n---\n\n*最后更新: 2024-06-25*\n*版本: 设计方案*\n*状态: 正在进行中*