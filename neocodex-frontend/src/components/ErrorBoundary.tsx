import { ErrorBoundary as SolidErrorBoundary } from 'solid-js'
import type { Component, JSX } from 'solid-js'

/**
 * 全局错误边界 — 渲染期 panic 兜底, 避免白屏。
 * 包裹 Router, 捕获 Chat/GlobeView 等懒加载组件的渲染异常。
 */
export const ErrorBoundary: Component<{ children?: JSX.Element }> = (props) => {
  return (
    <SolidErrorBoundary
      fallback={(err, reset) => (
        <div class="flex h-full min-h-[100vh] items-center justify-center bg-bg-primary p-8">
          <div class="max-w-lg rounded-lg border border-border-color bg-bg-secondary p-6">
            <div class="mb-2 text-[15px] font-semibold text-text-primary">界面渲染出错</div>
            <div class="mb-4 overflow-auto text-[13px] text-text-muted">
              <pre class="whitespace-pre-wrap break-all font-mono">{String(err)}</pre>
            </div>
            <button
              onClick={() => reset()}
              class="rounded-md bg-accent-primary px-4 py-2 text-[13px] font-medium text-white transition-colors hover:bg-accent-primary/90"
            >
              重试
            </button>
          </div>
        </div>
      )}
    >
      {props.children}
    </SolidErrorBoundary>
  )
}