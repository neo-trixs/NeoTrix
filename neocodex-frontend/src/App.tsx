import { lazy, Suspense } from 'solid-js'
import { Router, Route } from '@solidjs/router'
import { TrafficLights } from './components/TrafficLights'
import { ErrorBoundary } from './components/ErrorBoundary'
import { PageShortcuts } from './components/PageShortcuts'

// 代码分割：重组件（Chat / 3D GlobeView）按需懒加载，配合 vite manualChunks 分包
const Chat = lazy(() => import('./routes/Chat').then((m) => ({ default: m.Chat })))
const GlobeView = lazy(() => import('./components/GlobeView').then((m) => ({ default: m.GlobeView })))
const KnowledgeBase = lazy(() => import('./routes/KnowledgeBase').then((m) => ({ default: m.KnowledgeBase })))
const Marketplace = lazy(() => import('./routes/Marketplace').then((m) => ({ default: m.Marketplace })))
const Insights = lazy(() => import('./routes/Insights').then((m) => ({ default: m.Insights })))
const Skills = lazy(() => import('./routes/Skills').then((m) => ({ default: m.Skills })))
const MemoryManager = lazy(() => import('./routes/MemoryManager').then((m) => ({ default: m.MemoryManager })))
const Workflows = lazy(() => import('./routes/Workflows').then((m) => ({ default: m.Workflows })))

function GlobeRoute() {
  return <GlobeView limit={5000} height={700} />
}

export default function App() {
  return (
    <>
      <TrafficLights />
      <Suspense
        fallback={
          <div
            class="fixed inset-0 flex items-center justify-center bg-bg-primary/60"
            style={{ 'backdrop-filter': 'blur(2px)' }}
          >
            <span class="text-[13px] text-text-muted">加载中…</span>
          </div>
        }
      >
        <ErrorBoundary>
          <Router>
            <Route path="/" component={Chat} />
            <Route path="/chat" component={Chat} />
            <Route path="/globe" component={GlobeRoute} />
            <Route path="/kb" component={KnowledgeBase} />
            <Route path="/plugins" component={Marketplace} />
            <Route path="/insights" component={Insights} />
            <Route path="/skills" component={Skills} />
            <Route path="/memory" component={MemoryManager} />
            <Route path="/workflows" component={Workflows} />
            {/* ⌘1..7 页面快捷键 — catch-all 最低优先级, 返回 null */}
            <Route path="*" component={PageShortcuts} />
          </Router>
        </ErrorBoundary>
      </Suspense>
    </>
  )
}