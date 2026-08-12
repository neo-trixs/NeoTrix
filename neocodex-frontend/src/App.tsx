import { lazy, Suspense } from 'solid-js'
import { Router, Route } from '@solidjs/router'
import { TrafficLights } from './components/TrafficLights'

// 代码分割：重组件（Chat / 3D GlobeView）按需懒加载，配合 vite manualChunks 分包
const Chat = lazy(() => import('./routes/Chat').then((m) => ({ default: m.Chat })))
const GlobeView = lazy(() => import('./components/GlobeView').then((m) => ({ default: m.GlobeView })))

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
        <Router>
          <Route path="/" component={Chat} />
          <Route path="/chat" component={Chat} />
          <Route path="/globe" component={GlobeRoute} />
        </Router>
      </Suspense>
    </>
  )
}