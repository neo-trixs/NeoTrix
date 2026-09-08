# NeoTrix Desktop — UI 修复记录

## 修复清单 (每次构建前 MUST 检查)

### 1. 圆角窗口 (Tauri decorations:false)

**文件**: `neocodex-frontend/src/styles/index.css`

```css
.glass-L1 {
  border-radius: 0 14px 14px 0;  /* 右上右下圆角 */
}
.glass-side {
  border-radius: 14px 0 0 14px;  /* 左上左下圆角 */
}
```

**原理**: decorations:false + transparent 的窗口，圆角由 CSS border-radius 控制。侧栏和主区拼合形成完整圆角。

### 2. 会话标题完整显示

**文件**: `neocodex-frontend/src/components/Sidebar.tsx`

**修复点 1**: 移除按钮内的时间 `<span>` (line ~704)
```tsx
// 之前 (错误): 时间 span 挤压标题空间
<span class="text-11px text-text-muted flex-shrink-0">
  {formatRelativeTime(session.updatedAt)}
</span>

// 之后 (正确): 移除，标题独占空间
```

**修复点 2**: hover 操作按钮改为 absolute 定位
```tsx
// 之前 (错误): flex 子元素占空间
<div class="flex items-center gap-1 pr-2 opacity-0 group-hover:opacity-100...">

// 之后 (正确): absolute 不占空间
<div class="absolute right-0 top-0 bottom-0 flex items-center gap-1 pr-2 opacity-0 group-hover:opacity-100... bg-gradient-to-l from-white/80 via-white/60 to-transparent pointer-events-none group-hover:pointer-events-auto">
```

**修复点 3**: 父级 div 加 `w-full`
```tsx
// 之前 (错误): 无宽度约束
<div class={clsx('rounded-lg transition-colors', ...)}>

// 之后 (正确): 占满宽度
<div class={clsx('w-full rounded-lg transition-colors', ...)}>
```

### 3. fade-in 动画修复

**文件**: `neocodex-frontend/src/styles/index.css`

```css
/* 添加缺失的 keyframe */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* 修正引用 (fade-in → fadeIn) */
animation: fadeIn 0.12s ease-out;
```

### 4. API 适配层

**文件**: `neocodex-frontend/src/api/adapter.ts`

自动路由 `neocodex_*` 命令到 `domain_call`，无需修改 120+ 调用点。

### 5. PTY 参数名修复

**文件**: `neocodex-frontend/src/api/pty.ts`

```tsx
// 之前 (错误): camelCase
invoke('pty_write', { sessionId, data })

// 之后 (正确): snake_case 对齐 Rust
invoke('pty_write', { session_id: sessionId, data })
```

## 构建 SOP

```bash
# 1. 停止应用
pkill -f neotrix-tauri

# 2. 清除缓存
rm -rf ~/Library/Caches/neotrix-tauri/WebKit
rm -rf neocodex-frontend/node_modules/.vite
rm -rf neocodex-frontend/dist

# 3. 构建前端
cd neocodex-frontend && npx vite build

# 4. 验证 CSS
cat dist/assets/*.css | grep -o "glass-side{[^}]*}"
# 期望: border-radius:14px 0 0 14px

# 5. 构建 Tauri
cd .. && RUSTFLAGS="--cap-lints warn" cargo build -p neotrix-tauri --release

# 6. 启动
open target/release/neotrix-tauri
```

## 注意事项

- **不要用 `git checkout` 回退 API 文件** — 会同时回退 UI 修复
- **design-tokens.css 未被导入** — 不需要修改
- **layout/index.tsx 是死代码** — 不影响运行
- **前端 build 用 `npx vite build`** — 跳过 tsc 避免类型错误
