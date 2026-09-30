import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

/**
 * 壳层自己（不是 iframe 里的网页）要跨源 POST 本地实例的 `/api-ssh`——远端机器
 * 切换器的秒级轮询走这条路。`connect-src` 漏掉本地实例来源时，fetch 被文档 CSP
 * 直接拒掉，控制台只刷「Refused to connect because it violates the document's
 * Content Security Policy」，切换器静默降级，功能整个不可用。端口因占用冲突会
 * 自动递增（见后端 runtime info），所以只能按 host 通配。
 */
const config = JSON.parse(
  readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'),
) as { app: { security: Record<'csp' | 'devCsp', string> } }

const serviceOrigin = /DSH_HOST: &str = "([^"]+)"/.exec(
  readFileSync(new URL('../src-tauri/src/config/constants.rs', import.meta.url), 'utf8'),
)?.[1]

describe('shell CSP loopback data plane', () => {
  it('reads the loopback service host from the Rust constants', () => {
    expect(serviceOrigin).toBe('http://127.0.0.1')
  })

  it.each(['csp', 'devCsp'] as const)('%s lets the shell fetch the local dsh instance', (key) => {
    const connectSrc = /(?:^|; )connect-src ([^;]+)/.exec(config.app.security[key])?.[1] ?? ''
    expect(connectSrc).toContain(`${serviceOrigin}:*`)
  })
})
