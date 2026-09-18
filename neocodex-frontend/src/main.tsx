import { render } from 'solid-js/web'
import App from './App'
import { initErrorMonitor } from './lib/errorMonitor'
import './styles/index.css'

initErrorMonitor()

if (import.meta.env.DEV) {
  const observer = new PerformanceObserver((list) => {
    for (const entry of list.getEntries()) {
      if (entry.duration > 100) {
        console.warn(`[perf] ${entry.name} took ${entry.duration.toFixed(1)}ms`)
      }
    }
  })
  observer.observe({ entryTypes: ['measure'] })
}

render(() => <App />, document.getElementById('root')!)