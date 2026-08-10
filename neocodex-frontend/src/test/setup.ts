// Vitest setup for SolidJS component tests.
// jsdom environment. jest-dom matchers are intentionally NOT imported here to
// keep tsc clean (its ambient jest types collide with vitest); use plain
// DOM assertions in tests instead.

// jsdom 的 localStorage 无 Storage 实现（get/set/clear 缺失），注入最小 mock。
// chatStore / tagsStore 依赖安全的 localStorage 持久化。
const lsStore = new Map<string, string>()
const stubStorage: Storage = {
  get length() { return lsStore.size },
  clear: () => lsStore.clear(),
  getItem: (k: string) => lsStore.has(k) ? lsStore.get(k)! : null,
  key: (i: number) => [...lsStore.keys()][i] ?? null,
  removeItem: (k: string) => { lsStore.delete(k) },
  setItem: (k: string, v: string) => { lsStore.set(k, String(v)) },
}

if (typeof globalThis !== 'undefined') {
  Object.defineProperty(globalThis, 'localStorage', {
    value: stubStorage,
    writable: true,
    configurable: true,
  })
  Object.defineProperty(window, 'localStorage', {
    value: stubStorage,
    writable: true,
    configurable: true,
  })
}
