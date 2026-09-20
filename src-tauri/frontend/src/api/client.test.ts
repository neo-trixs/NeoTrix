import { describe, it, expect, vi, beforeEach } from 'vitest'

const invokeMock = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}))

import { ApiError, call, callOr, errText, toApiError } from './client'

function catchErr(p: Promise<unknown>): Promise<unknown> {
  return p.then(
    () => {
      throw new Error('expected rejection')
    },
    (e: unknown) => e,
  )
}

describe('api/client — 错误信封三要素 (code/message/details)', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('call 成功透传结果', async () => {
    invokeMock.mockResolvedValue({ ok: true })
    await expect(call<{ ok: boolean }>('cmd')).resolves.toEqual({ ok: true })
    expect(invokeMock).toHaveBeenCalledWith('cmd', undefined)
  })

  it('call 失败归一为 ApiError（透传 message）', async () => {
    invokeMock.mockRejectedValue(new Error('boom'))
    const err = await catchErr(call('cmd'))
    expect(err).toBeInstanceOf(ApiError)
    expect((err as ApiError).message).toBe('boom')
    expect((err as ApiError).code).toBeNull()
  })

  it('invoke 拒绝值为 {code,message} 时提取语义码', async () => {
    invokeMock.mockRejectedValue({ code: 'ERR_PROVIDER_BUSY', message: 'rate limited', details: { retryIn: 5 } })
    const err = await catchErr(call('cmd'))
    expect(err).toBeInstanceOf(ApiError)
    expect((err as ApiError).message).toBe('rate limited')
    expect((err as ApiError).code).toBe('ERR_PROVIDER_BUSY')
    expect((err as ApiError).details).toEqual({ retryIn: 5 })
  })

  it('callOr 失败返回 fallback（静默）', async () => {
    invokeMock.mockRejectedValue(new Error('x'))
    await expect(callOr('cmd', {}, 'fb')).resolves.toBe('fb')
  })

  it('toApiError 幂等：已是 ApiError 原样返回', () => {
    const a = new ApiError('m', 'CODE')
    expect(toApiError(a)).toBe(a)
  })

  it('errText 优先 ApiError.message（含结构化错误）', () => {
    expect(errText(new ApiError('m', 'CODE'))).toBe('m')
    expect(errText(new Error('plain'))).toBe('plain')
    expect(errText('raw')).toBe('raw')
  })
})
