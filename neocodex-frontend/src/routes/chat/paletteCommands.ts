/* ════════════════════════════════════════════
   routes/chat/paletteCommands.ts — ⌘K 命令注册表
   从 Chat.tsx 外移 (拆分 spike 第一刀): 纯数据工厂,
   依赖经参数注入 — 命令定义与宿主解耦。
   ════════════════════════════════════════════ */
import type { PaletteCommand } from '../../components/CommandPalette'

export interface PaletteDeps {
  addSession(): void
  clearMessages(): void
  runCompact(): void
  navigate(to: string): void
  cyclePermissionMode(): void
  openHelp(): void
  openSettings(): void
  unifiedCliCmds(): PaletteCommand[]
}

export function buildPaletteCommands(d: PaletteDeps): PaletteCommand[] {
  return [
    { id: 'new', label: '新建对话', desc: '开启一段新对话', keywords: ['new', '新建', '对话'], run: () => d.addSession() },
    { id: 'clear', label: '清除会话', desc: '清空当前会话全部消息', keywords: ['clear', '清除', '清空'], run: () => d.clearMessages() },
    { id: 'compact', label: '压缩会话', desc: '精简上下文继续对话', keywords: ['compact', '压缩'], run: () => d.runCompact() },
    // ── 页面动作 (与 ⌘1..7 同源) ──
    { id: 'page-kb', label: '打开知识库', desc: '文档库管理与检索', keywords: ['kb', '知识库', '页面'], run: () => d.navigate('/kb') },
    { id: 'page-plugins', label: '打开插件市场', desc: '插件安装与管理', keywords: ['plugins', '插件', '市场'], run: () => d.navigate('/plugins') },
    { id: 'page-insights', label: '打开洞察', desc: '成本与活动仪表盘', keywords: ['insights', '洞察', '成本'], run: () => d.navigate('/insights') },
    { id: 'page-skills', label: '打开技能中心', desc: '已安装技能浏览与搜索', keywords: ['skills', '技能'], run: () => d.navigate('/skills') },
    { id: 'page-memory', label: '打开记忆管理', desc: '记忆统计/时间线/搜索', keywords: ['memory', '记忆'], run: () => d.navigate('/memory') },
    { id: 'page-workflows', label: '打开工作流', desc: '工作流列表与运行', keywords: ['workflow', '工作流', '流程'], run: () => d.navigate('/workflows') },
    { id: 'mode', label: '切换权限模式', desc: '自动 / 手动 / 接受编辑 / 规划', keywords: ['mode', '权限', '模式'], run: () => d.cyclePermissionMode() },
    { id: 'help', label: '快捷键帮助', desc: '显示常用快捷键说明', keywords: ['help', '帮助', '快捷键'], run: () => d.openHelp() },
    { id: 'settings', label: '打开设置', desc: '提供商配置与应用设置', keywords: ['settings', '设置', '配置'], run: () => d.openSettings() },
    ...d.unifiedCliCmds(),
  ]
}
