/**
 * ComponentInteractionTest — 组件交互测试
 * 
 * 在浏览器中运行，验证每个设置标签页的交互功能
 */
import { createSignal, onMount, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { domain } from '../../api'
import * as modelPool from '../../api/model-pool'
import * as proxyPool from '../../api/proxy-pool'
import * as im from '../../api/im'
import * as market from '../../api/market'

interface TestResult {
  name: string
  status: 'pass' | 'fail' | 'skip'
  message: string
  duration: number
}

interface TestSuite {
  name: string
  tests: TestResult[]
}

export function ComponentInteractionTest() {
  const [results, setResults] = createSignal<TestSuite[]>([])
  const [running, setRunning] = createSignal(false)
  const [currentTest, setCurrentTest] = createSignal<string>('')

  const runTest = async (
    name: string,
    fn: () => Promise<void>
  ): Promise<TestResult> => {
    const start = Date.now()
    try {
      await fn()
      return {
        name,
        status: 'pass',
        message: 'OK',
        duration: Date.now() - start,
      }
    } catch (e) {
      return {
        name,
        status: 'fail',
        message: e instanceof Error ? e.message : String(e),
        duration: Date.now() - start,
      }
    }
  }

  const runAllTests = async () => {
    setRunning(true)
    setResults([])
    
    const suites: TestSuite[] = []
    
    // 测试 1: 域系统
    setCurrentTest('域系统')
    const domainTests = await Promise.all([
      runTest('domain.list', async () => {
        const domains = await domain.list()
        if (!Array.isArray(domains)) throw new Error('Expected array')
      }),
      runTest('domain.has(session)', async () => {
        const has = await domain.has('session')
        if (!has) throw new Error('Session domain not found')
      }),
      runTest('domain.actionCount(session)', async () => {
        const count = await domain.actionCount('session')
        if (count <= 0) throw new Error('No actions')
      }),
    ])
    suites.push({ name: '域系统', tests: domainTests })

    // 测试 2: 模型池
    setCurrentTest('模型池')
    const modelTests = await Promise.all([
      runTest('modelPool.status', async () => {
        const status = await modelPool.getModelPoolStatus()
        if (!status) throw new Error('No status')
      }),
    ])
    suites.push({ name: '模型池', tests: modelTests })

    // 测试 3: 代理池
    setCurrentTest('代理池')
    const proxyTests = await Promise.all([
      runTest('proxyPool.status', async () => {
        const status = await proxyPool.getProxyPoolStatus()
        if (!status) throw new Error('No status')
      }),
    ])
    suites.push({ name: '代理池', tests: proxyTests })

    // 测试 4: IM
    setCurrentTest('IM')
    const imTests = await Promise.all([
      runTest('im.status', async () => {
        const status = await im.getImStatus()
        if (!status) throw new Error('No status')
      }),
      runTest('im.channels', async () => {
        const channels = await im.listChannels()
        if (!Array.isArray(channels)) throw new Error('Expected array')
      }),
    ])
    suites.push({ name: 'IM', tests: imTests })

    // 测试 5: 市场
    setCurrentTest('市场')
    const marketTests = await Promise.all([
      runTest('market.status', async () => {
        const status = await market.marketStatus()
        if (!status) throw new Error('No status')
      }),
    ])
    suites.push({ name: '市场', tests: marketTests })

    setResults(suites)
    setRunning(false)
    setCurrentTest('')
  }

  const totalTests = results().reduce((sum, s) => sum + s.tests.length, 0)
  const passedTests = results().reduce(
    (sum, s) => sum + s.tests.filter(t => t.status === 'pass').length,
    0
  )
  const failedTests = results().reduce(
    (sum, s) => sum + s.tests.filter(t => t.status === 'fail').length,
    0
  )

  return (
    <div class="space-y-4">
      <div class="ss-card">
        <div class="ss-card-header">
          <span class="text-base mr-2">🧪</span>
          组件交互测试
        </div>
        <div class="ss-card-body">
          <div class="flex items-center justify-between mb-4">
            <div class="text-sm text-text-secondary">
              验证所有设置标签页的 API 连接和组件交互
            </div>
            <button
              class={clsx(
                'px-4 py-2 rounded-lg text-sm font-medium transition-colors',
                running()
                  ? 'bg-zinc-100 text-zinc-400 cursor-not-allowed'
                  : 'bg-nt-io-500 text-white hover:bg-nt-io-600'
              )}
              onClick={runAllTests}
              disabled={running()}
            >
              {running() ? `测试中: ${currentTest()}...` : '运行测试'}
            </button>
          </div>

          <Show when={results().length > 0}>
            {/* 概览 */}
            <div class="grid grid-cols-3 gap-3 mb-4">
              <div class="p-3 rounded-xl bg-bg-primary/40 text-center">
                <div class="text-2xl font-bold text-text-primary">{totalTests}</div>
                <div class="text-[10px] text-text-muted">总测试数</div>
              </div>
              <div class="p-3 rounded-xl bg-emerald-50/50 text-center">
                <div class="text-2xl font-bold text-emerald-600">{passedTests}</div>
                <div class="text-[10px] text-emerald-600">通过</div>
              </div>
              <div class="p-3 rounded-xl bg-red-50/50 text-center">
                <div class="text-2xl font-bold text-red-600">{failedTests}</div>
                <div class="text-[10px] text-red-600">失败</div>
              </div>
            </div>

            {/* 详细结果 */}
            <div class="space-y-3">
              <For each={results()}>
                {(suite) => (
                  <div class="border border-border-primary/40 rounded-xl overflow-hidden">
                    <div class="px-3 py-2 bg-bg-primary/40 flex items-center justify-between">
                      <span class="text-sm font-medium text-text-primary">{suite.name}</span>
                      <span class="text-[10px] text-text-muted">
                        {suite.tests.filter(t => t.status === 'pass').length}/{suite.tests.length}
                      </span>
                    </div>
                    <div class="divide-y divide-border-primary/20">
                      <For each={suite.tests}>
                        {(test) => (
                          <div class="px-3 py-2 flex items-center gap-2 text-xs">
                            <span class={clsx(
                              'w-2 h-2 rounded-full flex-shrink-0',
                              test.status === 'pass' ? 'bg-emerald-500' : 'bg-red-500'
                            )} />
                            <span class="flex-1 font-mono text-text-primary">{test.name}</span>
                            <span class="text-text-muted">{test.duration}ms</span>
                            {test.status === 'fail' && (
                              <span class="text-red-500 truncate max-w-[200px]" title={test.message}>
                                {test.message}
                              </span>
                            )}
                          </div>
                        )}
                      </For>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </div>
  )
}
