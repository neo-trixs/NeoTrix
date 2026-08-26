/* ════════════════════════════════════════════
   routes/Marketplace.tsx — 插件市场页

   R-P42 复用：PluginMarketplace 组件自带 embedded 模式
   （无浮层/无关闭按钮），页面化直接复用，数据走既有
   api/plugins.ts（已接后端 plugin_* 命令）。
   ════════════════════════════════════════════ */
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, Store } from 'lucide-solid'
import { PluginMarketplace } from '../components/PluginMarketplace'

export function Marketplace() {
  const navigate = useNavigate()
  return (
    <div class="min-h-screen bg-bg-primary text-text-primary flex flex-col">
      <header class="flex items-center gap-3 px-5 h-12 border-b border-border-primary/40 shrink-0">
        <button
          class="flex items-center gap-1.5 text-13px text-text-muted hover:text-text-primary transition-colors"
          onClick={() => navigate('/chat')}
          aria-label="返回对话"
        >
          <ArrowLeft class="w-4 h-4" />
          对话
        </button>
        <h1 class="text-14px font-semibold flex items-center gap-1.5">
          <Store class="w-4 h-4 text-nt-io-600" />
          插件市场
        </h1>
      </header>
      <main class="flex-1 overflow-y-auto">
        <PluginMarketplace open embedded onClose={() => navigate('/chat')} />
      </main>
    </div>
  )
}
