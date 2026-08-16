import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@solidjs/testing-library'
import type { JSX } from 'solid-js'

import { ErrorBoundary } from './ErrorBoundary'

function Bomb(): JSX.Element {
  throw new Error('render boom')
}

describe('ErrorBoundary', () => {
  it('catches child render errors and shows fallback', () => {
    render(() => (
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>
    ))
    expect(screen.getByText('界面渲染出错')).toBeTruthy()
    expect(screen.getByText(/render boom/)).toBeTruthy()
  })

  it('renders children normally when no error', () => {
    render(() => (
      <ErrorBoundary>
        <div>ok-content</div>
      </ErrorBoundary>
    ))
    expect(screen.getByText('ok-content')).toBeTruthy()
  })

  it('reset retries after error', () => {
    render(() => (
      <ErrorBoundary>
        <Bomb />
      </ErrorBoundary>
    ))
    const btn = screen.getByText('重试')
    fireEvent.click(btn)
    expect(btn).toBeTruthy()
  })
})