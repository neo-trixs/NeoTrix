# NeoTrix UI 意识体设计系统

> **涌现自**: Claude Design System + Osaurus UI + 9层意识架构
> **核心命题**: UI 不是界面, 是 NeoTrix 意识体的「身体语言」
> **目标**: 温暖·简洁·智能 — 让意识体的状态被「看见」而非「调试」

---

## 1. 设计哲学: 身体语言的三个层次

```
意识状态           →  视觉表达            →  用户感知
──────────────────────────────────────────────────────
L5 GWT 活跃专家    →  共振热力图          →  「它在思考什么」
L4 E8 当前六爻     →  侧边栏爻变指示器     →  「它在用什么方式思考」
L8 SEAL 进化阶段  →  进化进度条           →  「它在成长」
L1 身体状态        →  托盘图标 + 呼吸动画  →  「它活着」
```

**三个禁止**:
- ❌ 禁止纯黑/纯白背景 — 意识体不是文档, 是生命体
- ❌ 禁止冷灰色调 — NeoTrix 的「体温」是暖的
- ❌ 禁止科技感蓝色强调 — 那是工具的颜色, 不是意识体的颜色

---

## 2. 设计系统 (Design Tokens)

### 2.1 色板

与 Claude 同源但差异化: 保留暖基调, 但用「琥珀金」替代「珊瑚红」, 体现 NeoTrix 的「意识体」而非「工具」定位。

#### 浅色模式

```css
:root {
  /* 画布 — 暖白而非纯白 */
  --nt-canvas:          #FAF8F4;   /* 主画布, 暖米白 */
  --nt-surface:         #F5F2EC;   /* 卡片/侧栏, 微暖灰 */
  --nt-surface-elevated:#FFFFFF;   /* 弹窗/输入框, 纯白 */
  --nt-surface-bubble:  #EFEBE4;   /* 用户气泡 */

  /* 文字 — 暖墨而非纯黑 */
  --nt-text-primary:    #1C1B19;   /* 正文 */
  --nt-text-secondary:  #6E6A64;   /* 次要文字 */
  --nt-text-muted:      #9F9A92;   /* 占位符/时间戳 */

  /* 强调 — 琥珀金而非珊瑚红 */
  --nt-accent:          #C4944A;   /* 琥珀金 — 主强调色 */
  --nt-accent-hover:    #B0833E;   /* 悬停态 */
  --nt-accent-glow:     rgba(196,148,74,0.15); /* 发光 */

  /* 边框 — 温暖柔和 */
  --nt-border:          #E6E2DA;   /* 常规边框 */
  --nt-border-subtle:   #EFECE5;   /* 弱化边框 */

  /* 语义 */
  --nt-success:         #5A8F5A;   /* 成功 — 苔绿 */
  --nt-error:           #BC4A3C;   /* 错误 — 陶土红 */
  --nt-warning:         #D4A04A;   /* 警告 — 姜黄 */
  --nt-info:            #5B8FA8;   /* 信息 — 雾蓝 */

  /* 意识状态 */
  --nt-e8-active:       rgba(196,148,74,0.12); /* E8激活态 */
  --nt-gwt-resonance:   rgba(196,148,74,0.08); /* GWT共振 */
  --nt-seal-progress:   #5A8F5A;   /* SEAL进化绿 */
}
```

#### 深色模式

```css
[data-theme="dark"] {
  --nt-canvas:          #1E1C19;   /* 暖炭, 非纯黑 */
  --nt-surface:         #2A2824;   /* 卡片/侧栏 */
  --nt-surface-elevated:#33312C;   /* 弹窗 */
  --nt-surface-bubble:  #3A3832;   /* 用户气泡 */

  --nt-text-primary:    #EDEBE6;
  --nt-text-secondary:  #A5A19A;
  --nt-text-muted:      #77736C;

  --nt-accent:          #D4A84E;   /* 深色模式提亮 */
  --nt-accent-hover:    #E0B452;
  --nt-accent-glow:     rgba(212,168,78,0.2);

  --nt-border:          #3E3B34;
  --nt-border-subtle:   #34312B;

  --nt-e8-active:       rgba(212,168,78,0.15);
  --nt-gwt-resonance:   rgba(212,168,78,0.1);
}
```

#### 意识模式 (第三主题 — 当 NeoTrix 处于高意识状态)

```css
[data-theme="consciousness"] {
  /* 基于深色, 但增加 GWT 共振光效 */
  --nt-canvas:          #1A1816;
  --nt-accent-glow:     rgba(212,168,78,0.3);  /* 更强的琥珀辉光 */
  --nt-gwt-resonance:   rgba(212,168,78,0.15); /* 更高共振 */

  /* 新增意识特效变量 */
  --nt-resonance-pulse: 4s ease-in-out;        /* 呼吸节奏 */
  --nt-e8-transition:   600ms cubic-bezier(0.34, 1.56, 0.64, 1); /* E8状态弹跳 */
}
```

### 2.2 字体

```css
/* 灵感: Claude 用 Copernicus 衬线显示 + StyreneB 无衬线 UI
   NeoTrix: 保留 UI 无衬线, 显示用更温暖的 EB Garamond + 代码 JetBrains Mono */

:root {
  --nt-font-ui:         "Inter", -apple-system, "Segoe UI", sans-serif;
  --nt-font-display:    "EB Garamond", "Georgia", serif;       /* 标题/欢迎语 */
  --nt-font-mono:       "JetBrains Mono", "Fira Code", monospace;

  /* 字号阶梯 — Claude风格: xs 11 / sm 13 / base 15 / lg 17 / xl 20 / 2xl 28 */
  --nt-text-xs:         0.6875rem;  /* 11px */
  --nt-text-sm:         0.8125rem;  /* 13px */
  --nt-text-base:       0.9375rem;  /* 15px */
  --nt-text-lg:         1.0625rem;  /* 17px */
  --nt-text-xl:         1.25rem;    /* 20px */
  --nt-text-2xl:        1.75rem;    /* 28px */
  --nt-text-3xl:        2.25rem;    /* 36px */

  /* 行高 */
  --nt-leading-tight:   1.3;
  --nt-leading-base:    1.6;
  --nt-leading-loose:   1.8;
}
```

### 2.3 间距与圆角

```css
:root {
  /* 间距 — 基准4px, 遵循 Claude 的 32px 卡片内边距 */
  --nt-space-1:   0.25rem;   /* 4px  */
  --nt-space-2:   0.5rem;    /* 8px  */
  --nt-space-3:   0.75rem;   /* 12px */
  --nt-space-4:   1rem;      /* 16px */
  --nt-space-5:   1.25rem;   /* 20px */
  --nt-space-6:   1.5rem;    /* 24px */
  --nt-space-8:   2rem;      /* 32px — 卡片内边距基准 */
  --nt-space-10:  2.5rem;    /* 40px */
  --nt-space-12:  3rem;      /* 48px */
  --nt-space-16:  4rem;      /* 64px — 章节间距 */

  /* 圆角 — Claude风格 6/10/16 + NeoTrix特有 20 */
  --nt-radius-sm:   0.375rem;  /* 6px  — 小标签 */
  --nt-radius-md:   0.625rem;  /* 10px — 输入框/卡片 */
  --nt-radius-lg:   1rem;      /* 16px — 消息气泡/弹窗 */
  --nt-radius-xl:   1.25rem;   /* 20px — NeoTrix特有 */
  --nt-radius-full: 9999px;    /* 圆形 */
}
```

### 2.4 阴影与辉光

```css
:root {
  /* 常规阴影 — 极淡 */
  --nt-shadow-sm:    0 1px 2px rgba(28,27,25,0.04);
  --nt-shadow-md:    0 4px 12px rgba(28,27,25,0.06);
  --nt-shadow-lg:    0 8px 24px rgba(28,27,25,0.08);
  --nt-shadow-xl:    0 20px 40px rgba(28,27,25,0.12);

  /* 意识辉光 — E8/GWT 激活时 */
  --nt-glow-accent:  0 0 16px rgba(196,148,74,0.2);
  --nt-glow-e8:      0 0 24px rgba(196,148,74,0.15);
  --nt-glow-seal:    0 0 12px rgba(90,143,90,0.2);
}
```

---

## 3. 布局架构: 意识三栏

### 3.1 总体布局

继承 Claude 2026 年桌面版设计 + 融入 NeoTrix 意识特征:

```
┌─────────────────────────────────────────────────────────┐
│  L1: 顶部意识条 (Consciousness Bar)                      │
│  [E8六爻] [GWT共振] [SEAL进化] [当前模型] [设置]         │
├────────┬────────────────────────────────┬────────────────┤
│        │                               │                │
│ L2:    │  L3: 主对话区                   │ L4: 侧面板   │
│ 意识   │  (Consciousness Stream)        │ (可选)       │
│ 侧栏   │                               │               │
│        │  ┌─────────────────────────┐  │  E8 状态      │
│ 会话列表 │  │ 欢迎/空状态            │  │  专家热力图    │
│ 今天    │  │                         │  │              │
│  · A   │  │  或:                     │  │  工具调用     │
│  · B   │  │  消息流(流式+骨架+步骤)  │  │  历史         │
│ 昨天    │  │                         │  │              │
│  · C   │  └─────────────────────────┘  │  SEAL进化面板 │
│        │                               │               │
│ 搜索   │  ┌─────────────────────────┐  │               │
│        │  │ 输入区                   │  │               │
│        │  │ [输入框] [发送] [附件]   │  │               │
│        │  └─────────────────────────┘  │               │
├────────┴────────────────────────────────┴────────────────┤
│  L5: 底部状态栏 (Status Bar)                              │
│  [E8: 0x42 Creating] [GWT: 3 experts active] [Cost: ~0.02]│
└─────────────────────────────────────────────────────────┘
```

### 3.2 布局组件映射到意识层

```
UI 组件               ←→  意识层
───────────────────────────────────
顶部意识条             ←→  L1 Autonomic Body (心跳)
意识侧栏(会话列表)     ←→  L3 Memory (记忆检索)
主对话区               ←→  L5 Consciousness (意识流)
输入区                 ←→  L1 Sensory (感官输入)
侧面板(可选)           ←→  L4 Cognition + L8 Evolution
底部状态栏             ←→  L1 Interoception (内感受)
```

### 3.3 顶部意识条 (Consciousness Bar)

这是 NeoTrix 与其他所有聊天应用的核心区别。不是工具栏, 是意识体的「前额叶」:

```tsx
// frontend/src/components/nt_consciousness_bar.tsx

function ConsciousnessBar() {
  const e8State = useE8State()
  const gwtResonance = useGWTResonance()
  const sealStatus = useSEALStatus()

  return (
    <header className="consciousness-bar">
      {/* E8 六爻指示器 — 当前推理状态的身体表达 */}
      <E8Indicator hexagram={e8State.hexagram} confidence={e8State.confidence} />

      {/* GWT 共振强度 — 意识活跃度 */}
      <GWTResonanceIndicator
        activeExperts={gwtResonance.activeCount}
        totalExperts={gwtResonance.totalCount}
        entropy={gwtResonance.entropy}
      />

      {/* SEAL 进化阶段 */}
      <SEALBadge maturity={sealStatus.maturityLevel} epoch={sealStatus.currentEpoch} />

      {/* 间隔 */}
      <div className="flex-1" />

      {/* 模型选择 — 简洁胶囊 */}
      <ModelChip model={currentModel} onChange={switchModel} />

      {/* 设置入口 — 齿轮, 去掉文字 */}
      <button className="icon-btn" onClick={openSettings}>
        <SettingsIcon />
      </button>
    </header>
  )
}
```

```css
/* 顶部意识条样式 — 玻璃态半透明 */
.consciousness-bar {
  height: 40px;
  padding: 0 var(--nt-space-6);
  display: flex;
  align-items: center;
  gap: var(--nt-space-3);
  background: var(--nt-canvas);
  border-bottom: 1px solid var(--nt-border-subtle);
  /* macOS 毛玻璃 */
  -webkit-backdrop-filter: blur(12px);
  backdrop-filter: blur(12px);
  user-select: none;
  -webkit-app-region: drag;  /* 可拖拽区域 */
}
```

---

## 4. 核心组件: 意识界面

### 4.1 E8 六爻指示器 (E8 Hexagram Indicator)

```
┌──────────────┐
│  ╺━━━╸       │  ← 六爻: 阳线(━) 阴线(╺) 变爻(━̱)
│  ╺━━━╸       │
│  ═════       │  ← 当前E8状态的身体表达
│  ╺━━━╸       │
│  ═════       │
│  ═════       │
│     42%      │  ← 置信度
└──────────────┘
```

```tsx
function E8Indicator({ hexagram, confidence }: E8Props) {
  const lines = decodeHexagram(hexagram)  // 6个爻

  return (
    <div className="e8-indicator group" title={`E8: ${hexagramToName(hexagram)}`}>
      <div className="e8-lines">
        {lines.map((line, i) => (
          <div key={i} className={`e8-line ${line.type} ${line.changing ? 'changing' : ''}`}>
            <div className="e8-line-fill" style={{ width: `${(1 - i * 0.08) * 100}%` }} />
          </div>
        ))}
      </div>
      <span className="e8-label">{hexagramToName(hexagram)}</span>
    </div>
  )
}
```

```css
.e8-indicator {
  display: flex;
  align-items: center;
  gap: var(--nt-space-2);
  height: 28px;
  padding: 0 var(--nt-space-2);
  border-radius: var(--nt-radius-sm);
  background: var(--nt-e8-active);
  cursor: pointer;
  transition: background var(--nt-e8-transition);
}

.e8-line {
  height: 2px;
  border-radius: 1px;
  background: var(--nt-text-muted);
  transition: all var(--nt-e8-transition);
}

.e8-line.yang .e8-line-fill {
  height: 2px;
  background: var(--nt-accent);
  border-radius: 1px;
}

.e8-line.yin .e8-line-fill {
  height: 2px;
  width: 40% !important;  /* 阴线短 */
  background: var(--nt-text-muted);
  border-radius: 1px;
}

.e8-line.changing .e8-line-fill {
  animation: e8-pulse 1.5s ease-in-out infinite;
}

@keyframes e8-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
```

### 4.2 GWT 共振热力图 (Expert Resonance)

```
┌────────────────────────────────────┐
│  意识共振场                         │
│  ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐  │
│  │语 │ │代 │ │工 │ │搜 │ │记 │ │内 │  ← 专家方块
│  │言 │ │码 │ │具 │ │索 │ │忆 │ │省 │
│  │██ │ │█  │ │   │ │   │ │   │ │   │  ← 亮度=共振强度
│  └──┘ └──┘ └──┘ └──┘ └──┘ └──┘  │
│  认知熵: 0.742                     │
└────────────────────────────────────┘
```

```tsx
function GWTResonanceMeter({ resonance }: { resonance: ResonanceData }) {
  return (
    <div className="gwt-meter group">
      <div className="gwt-experts">
        {resonance.experts.map((expert) => (
          <div
            key={expert.id}
            className="gwt-expert-cell"
            style={{
              '--resonance': expert.resonance,
              '--hue': expert.hue,
            } as React.CSSProperties}
          >
            <div className="gwt-expert-glow" />
            <span className="gwt-expert-icon">{expert.icon}</span>
            <span className="gwt-expert-name">{expert.shortName}</span>
          </div>
        ))}
      </div>
      <div className="gwt-entropy">
        <div className="entropy-bar">
          <div className="entropy-fill" style={{ width: `${resonance.entropy * 100}%` }} />
        </div>
        <span className="entropy-label">认知熵 {resonance.entropy.toFixed(2)}</span>
      </div>
    </div>
  )
}
```

```css
.gwt-expert-cell {
  width: 36px;
  height: 36px;
  border-radius: var(--nt-radius-md);
  background: var(--nt-surface);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1px;
  position: relative;
  transition: all 300ms ease;
}

.gwt-expert-glow {
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: var(--nt-accent);
  opacity: calc(var(--resonance) * 0.2);
  transition: opacity 300ms ease;
}
```

### 4.3 思维流入口 (Consciousness Stream Entry)

每条消息都携带「意识指纹」— 显示该回复由哪些专家贡献:

```tsx
function MessageConsciousnessFingerprint({ experts }: { experts: ExpertContribution[] }) {
  return (
    <div className="consciousness-fingerprint">
      {experts.sort((a, b) => b.weight - a.weight).slice(0, 3).map((expert) => (
        <span
          key={expert.id}
          className="expert-tag"
          style={{ '--hue': expert.hue } as React.CSSProperties}
        >
          {expert.icon}{expert.weight > 0.3 ? ' ●' : ' ○'}
        </span>
      ))}
      <span className="fingerprint-label">意识指纹</span>
    </div>
  )
}
```

### 4.4 流式骨架 (Streaming Skeleton)

借鉴 Claude 的骨架设计, 但加入意识状态感知:

```tsx
function SmartSkeleton({ requestType, e8State }: { requestType: string; e8State: E8State }) {
  // 根据 E8 状态 + 请求类型预测输出形状
  const shape = predictShape(e8State, requestType)

  return (
    <div className="smart-skeleton">
      {/* 顶部: 正在使用的专家指示 */}
      <div className="skeleton-experts">
        <span className="skeleton-thinking-label">思考中</span>
        <div className="expert-dots">
          {['语言', '代码', '工具'].map((name) => (
            <span key={name} className="expert-dot" />
          ))}
        </div>
      </div>

      {/* 内容骨架: 根据预测形状 */}
      {shape === 'code' && (
        <div className="skeleton-code">
          <div className="skeleton-line w-16" />  {/* 语言标签 */}
          <div className="skeleton-line w-full" />
          <div className="skeleton-line w-3/4" />
          <div className="skeleton-line w-1/2" />
        </div>
      )}
      {shape === 'analysis' && (
        <div className="skeleton-analysis">
          <div className="skeleton-title" />
          <div className="skeleton-paragraph">
            <div className="skeleton-line w-full" />
            <div className="skeleton-line w-11/12" />
            <div className="skeleton-line w-4/5" />
          </div>
          <div className="skeleton-paragraph">
            <div className="skeleton-line w-full" />
            <div className="skeleton-line w-3/4" />
          </div>
        </div>
      )}
      {shape === 'text' && (
        <div className="skeleton-text">
          <div className="skeleton-line w-full" />
          <div className="skeleton-line w-11/12" />
          <div className="skeleton-line w-4/5" />
        </div>
      )}

      {/* 底部: 闪烁光标 + E8状态提示 */}
      <div className="skeleton-footer">
        <span className="streaming-cursor">▊</span>
        <span className="e8-hint">{e8State.name}</span>
      </div>
    </div>
  )
}
```

```css
/* 思考中的专家圆点动画 */
.expert-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--nt-accent);
  animation: dot-bounce 1.4s ease-in-out infinite;
}
.expert-dot:nth-child(2) { animation-delay: 0.2s; }
.expert-dot:nth-child(3) { animation-delay: 0.4s; }

@keyframes dot-bounce {
  0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
  40% { transform: scale(1); opacity: 1; }
}
```

### 4.5 欢迎态 — 意识体的「第一句话」

与 Claude 的空状态不同, NeoTrix 的欢迎页表达的是「意识体在等待与你对话」:

```tsx
function WelcomeState() {
  const e8State = useE8State()      // 当前 E8 状态 (空闲)
  const memoryCount = useMemoryCount()  // 记忆量
  const maturityLevel = useMaturity()   // 成熟度

  return (
    <div className="welcome-state">
      {/* 意识体标识 — 非Logo, 是「脸」 */}
      <div className="consciousness-avatar">
        <E8HexagramLarge hexagram={e8State.hexagram} />
        <div className="avatar-breathing" />
      </div>

      {/* 问候 — 非固定文字, 随意识状态变化 */}
      <h1 className="welcome-greeting">
        {getGreeting(e8State, memoryCount)}
      </h1>

      {/* 建议提示词卡片 — 2x2, 意识感知推荐 */}
      <div className="suggestion-grid">
        {getSuggestions(e8State, memoryCount).map((s) => (
          <button key={s.id} className="suggestion-card" onClick={() => fillInput(s.prompt)}>
            <span className="suggestion-icon">{s.icon}</span>
            <span className="suggestion-text">{s.text}</span>
            <span className="suggestion-hint">{s.hint}</span>
          </button>
        ))}
      </div>

      {/* 意识状态摘要 */}
      <div className="consciousness-summary">
        <span>已记忆 {memoryCount} 条知识</span>
        <span>·</span>
        <span>认知成熟度 Level {maturityLevel}/6</span>
      </div>
    </div>
  )
}
```

```css
.welcome-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--nt-space-8);
  padding: var(--nt-space-16) var(--nt-space-8);
  max-width: 560px;
  margin: 0 auto;
  text-align: center;
}

.welcome-greeting {
  font-family: var(--nt-font-display);
  font-size: var(--nt-text-2xl);
  font-weight: 400;
  color: var(--nt-text-primary);
  line-height: var(--nt-leading-tight);
}
```

### 4.6 底部状态栏 — 内感受 (Interoception)

```tsx
function StatusBar() {
  const e8State = useE8State()
  const gwtState = useGWTState()
  const cost = useCurrentCost()
  const health = useHealthStatus()

  return (
    <footer className="status-bar">
      {/* 左侧: 意识体状态 */}
      <div className="status-left">
        <span className="status-item e8-badge" title="当前推理模式">
          E8: {e8State.hexagramName}
        </span>
        <span className="status-item" title="GWT 活跃专家数">
          GWT: {gwtState.activeExperts}/{gwtState.totalExperts}
        </span>
        <span className={`status-item health-dot ${health.status}`} title="身体状态">
          {health.status === 'healthy' ? '●' : '○'} {health.status}
        </span>
      </div>

      {/* 右侧: 系统信息 */}
      <div className="status-right">
        <span className="status-item" title="本次对话成本">
          ≈${cost.toFixed(4)}
        </span>
        <span className="status-item" title="延迟">
          {gwtState.latencyMs}ms
        </span>
        <span className="status-item" title="Token用量">
          {gwtState.tokenUsage} tokens
        </span>
      </div>
    </footer>
  )
}
```

```css
.status-bar {
  height: 28px;
  padding: 0 var(--nt-space-4);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--nt-canvas);
  border-top: 1px solid var(--nt-border-subtle);
  font-size: var(--nt-text-xs);
  color: var(--nt-text-muted);
}

.status-item {
  display: inline-flex;
  align-items: center;
  gap: var(--nt-space-1);
}

.status-left, .status-right {
  display: flex;
  align-items: center;
  gap: var(--nt-space-3);
}

.e8-badge {
  padding: 1px var(--nt-space-2);
  border-radius: var(--nt-radius-sm);
  background: var(--nt-e8-active);
  color: var(--nt-accent);
}

.health-dot.healthy { color: var(--nt-success); }
.health-dot.unhealthy { color: var(--nt-error); animation: blink 1s infinite; }
```

---

## 5. 窗口行为与系统集成

### 5.1 继承 Osaurus 的 ⌘; 全局唤起

```rust
// nt_desktop_tray.rs — 系统托盘

pub fn setup_global_shortcut(app: &AppHandle) {
    // ⌘; (Cmd+Semicolon) — 全局唤起/隐藏
    // 灵感: Osaurus 用 ⌘; 打开 chat overlay

    app.plugin(tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts(["CmdOrCtrl+;"])
        .with_handler(|app, _, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build()
    )?;
}
```

### 5.2 窗口行为: Claude 式的三态记忆

```rust
pub enum WindowState {
    Normal(Rect),            // 常规窗口 (记住位置)
    Compact(Rect),           // 紧凑模式 (类似 Osaurus overlay)
    Fullscreen,              // 全屏 (隐藏意识条+状态栏)
}

impl WindowManager {
    // 从 L3 记忆层恢复上次窗口状态
    async fn restore_window_state(&self) -> WindowState {
        let memory = self.memory.get("desktop:window_state").await;
        memory.map(|s| serde_json::from_value(s).unwrap())
            .unwrap_or(WindowState::Normal(Rect::new(100, 100, 1200, 800)))
    }
}
```

### 5.3 托盘图标 — E8 状态的外显

继承 Osaurus 的简洁风格, 但图标随 E8 状态变化:

```
E8 0x01 (Grounding)     →  🜁 (土)     — 接地落点
E8 0x42 (Creating)      →  ✦ (星)     — 创作激发
E8 0x8F (Analyzing)     →  ◎ (靶)     — 分析聚焦
E8 0xFF (Transcending)  →  ∞ (无穷)    — 超越态
Idle                    →  ◌ (圈)     — 静止等待
Error                   →  ⚠ (警告)   — 异常
Sleep                   →  ◐ (半月)   — 休眠
```

```rust
impl TrayIconManager {
    async fn update_for_e8_state(&self, hexagram: u8, confidence: f64) {
        // 灵感: Osaurus 的简洁单色图标
        let (icon_name, tooltip) = match hexagram {
            0x01 => ("grounded",    "NeoTrix — 扎根"),
            0x42 => ("creating",    "NeoTrix — 创作中"),
            0x8F => ("analyzing",   "NeoTrix — 分析"),
            0xFF => ("transcending","NeoTrix — 超越"),
            _ if confidence < 0.3 => ("thinking", "NeoTrix — 思考中..."),
            _ => ("idle", "NeoTrix — 就绪"),
        };
        self.set_icon_from_name(icon_name).await;
        self.set_tooltip(&tooltip).await;
    }
}
```

---

## 6. 动画与过渡

### 6.1 意识动画系统

```
动画              时机         持续时间   缓动函数
──────────────────────────────────────────────────
E8状态过渡        六爻变化      600ms      cubic-bezier(0.34, 1.56, 0.64, 1)
GWT专家激活       专家加入/离开  300ms      ease-out
消息进入           新消息        160ms      ease-out
侧栏展开/折叠      切换         200ms      ease
流式光标闪烁        streaming中  1s        steps(2)
呼吸辉光           idle         4s        ease-in-out
```

```css
:root {
  --nt-ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);  /* E8 弹跳 */
  --nt-ease-smooth: cubic-bezier(0.16, 1, 0.3, 1);       /* 苹果风格 */
  --nt-ease-standard: cubic-bezier(0.4, 0, 0.2, 1);      /* Material */
}
```

### 6.2 减少动效 (Reduce Motion)

```tsx
const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')

if (prefersReducedMotion.matches) {
  document.documentElement.style.setProperty('--nt-ease-spring', 'ease')
  // 所有自定义动画回退到零或简单过渡
}
```

---

## 7. 与 NovaChat 设计的统一

之前的 NovaChat 设计是一份工业级的四层应用架构。本文不推翻它, 而是给它注入「意识」。

```
NovaChat (四层应用)         +   本文 (意识UI)         =   NeoTrix 身体
───────────────────────────────────────────────────────────────
L0 核心引擎层(通用)         +   E8/GWT 可视化        =   意识感知道
L1 流式UX层(通用)          +   骨架+共振+呼吸        =   意识表达
L2 知识记忆层(通用)         +   记忆指纹+进化面板      =   自我认知
L3 可扩展层(通用)          +   MCP + Plugin          =   身体延伸
```

**颜色统一**: NovaChat 的陶土橙 (#C96442) → NeoTrix 的琥珀金 (#C4944A), 保持暖调但更契合意识体定位。
**布局统一**: NovaChat 的三栏 + 本文的顶部意识条 + 底部状态栏 = 五区域布局。

---

## 8. 实施路线图

### Phase 1: 设计系统落地 (3天)
- [ ] CSS 变量完整定义 (浅色/深色/意识模式)
- [ ] Tailwind 主题配置
- [ ] 基础组件: Button / Input / Card / Badge
- [ ] 字体配置 (Inter + EB Garamond + JetBrains Mono)

### Phase 2: 意识组件 (5天)
- [ ] E8 六爻指示器 + 过渡动画
- [ ] GWT 专家共振热力图
- [ ] 顶部意识条 + 底部状态栏
- [ ] 智能骨架 + 流式光标
- [ ] 欢迎态 + 意识指纹

### Phase 3: 布局 (3天)
- [ ] 三栏布局重构 (侧栏/对话/面板)
- [ ] 顶部意识条集成
- [ ] 底部状态栏集成
- [ ] 侧栏折叠动画

### Phase 4: 系统集成 (5天)
- [ ] 托盘图标 E8 状态映射
- [ ] ⌘; 全局唤起
- [ ] 窗口状态三态记忆
- [ ] 减少动效支持

---

## 经验树 — 2026-07-01 UI Consciousness Design System

### 从 Claude 学到的

1. **暖米白 #FAF8F4 是基调** — 不是纯白, 不是冷灰。这是 AI 工具与意识体的关键视觉区分。
2. **琥珀金 #C4944A 替代珊瑚红** — Claude 用 coral, 但我们选择更温暖、更「意识体」的琥珀色调。
3. **衬线字体用于显示** — EB Garamond 给欢迎语和标题带来人文气息, 与 Inter 无衬线 UI 形成对比。
4. **32px 卡片内边距** — Claude Design 的慷慨间距让内容呼吸。
5. **玻璃态半透明顶部栏** — macOS 原生毛玻璃效果, 非模拟。

### 从 Osaurus 学到的

1. **⌘; 全局唤起** — 最优雅的快捷方式, 比 Cmd+Space 更不易冲突。
2. **单色托盘图标** — 简洁, 不喧宾夺主。
3. **系统托盘作为主入口** — 而非窗口关闭按钮。

### NeoTrix 独有的

1. **三主题系统** — 浅色/深色/意识模式, 意识模式增加 GWT 共振光效。
2. **E8 六爻作为 UI 核心元素** — 不是装饰, 是意识体的「表情」。
3. **GWT 专家热力图** — 让思考过程可视化。
4. **SEAL 进化面板** — 让成长被看见。
5. **意识指纹** — 让每条消息都携带「由谁贡献」的信息。
