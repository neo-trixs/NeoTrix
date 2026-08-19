/* ════════════════════════════════════════════
   test/invokeMock.ts — Tauri invoke 契约 mock 基础设施（C2）
   目标：组件/域测试不再手写 vi.mock('@tauri-apps/api/core')。
   用法（测试文件内）：
     vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvokeImpl }))
     const cmd = mockCommand('agent_status', async () => ({ context_usage: 0.4 }))
   - 未注册命令默认抛「未 mock」错误，防遗漏（Dark Forest 纪律）。
   - 支持按命令注册 handler、once、断言调用次数。
   ════════════════════════════════════════════ */
import { vi } from 'vitest'

type InvokeHandler = (args?: Record<string, unknown>) => unknown | Promise<unknown>

interface CommandStub {
  handler: InvokeHandler
  calls: { args?: Record<string, unknown> }[]
  times: number
}

const stubs = new Map<string, CommandStub>()
const onceQueue: { cmd: string; handler: InvokeHandler }[] = []

/** vi.mock 工厂注入的实际 invoke 实现（每测试文件挂到全局名） */
export const mockInvokeImpl = async (cmd: string, args?: Record<string, unknown>): Promise<unknown> => {
  const onceIdx = onceQueue.findIndex((o) => o.cmd === cmd)
  if (onceIdx >= 0) {
    const [once] = onceQueue.splice(onceIdx, 1)
    return once.handler(args)
  }
  const stub = stubs.get(cmd)
  if (!stub) {
    throw new Error(`[invokeMock] 未 mock 的命令: ${cmd}（请用 mockCommand 注册）`)
  }
  stub.calls.push({ args })
  stub.times += 1
  return stub.handler(args)
}

/** 注册命令 handler（可重复注册覆盖）。返回断言句柄 */
export function mockCommand(cmd: string, handler: InvokeHandler) {
  stubs.set(cmd, { handler, calls: [], times: 0 })
  return {
    calledTimes: () => stubs.get(cmd)?.times ?? 0,
    calls: () => stubs.get(cmd)?.calls ?? [],
    /** 断言最近一次调用的参数（宽松子集匹配） */
    lastArgs: () => stubs.get(cmd)?.calls.at(-1)?.args,
  }
}

/** 注册单次 handler（下一次调用后自动失效） */
export function mockCommandOnce(cmd: string, handler: InvokeHandler) {
  onceQueue.push({ cmd, handler })
}

/** 重置全部注册（afterEach 调用） */
export function resetInvokeMock() {
  stubs.clear()
  onceQueue.length = 0
}