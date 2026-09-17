import { describe, it, expect } from 'vitest'
import { layoutNodes, bounds } from '../index'

describe('widgets/canvas-viewer', () => {
  it('layoutNodes 按网格布局', () => {
    const nodes = [{ id: 'a', x: 0, y: 0, label: 'A' }, { id: 'b', x: 0, y: 0, label: 'B' }, { id: 'c', x: 0, y: 0, label: 'C' }]
    const laid = layoutNodes(nodes, 100)
    expect(laid[0]).toEqual({ id: 'a', x: 0, y: 0, label: 'A' })
    expect(laid[1].x).toBe(100)
    expect(laid[2].x).toBe(200)
  })

  it('bounds 计算边界', () => {
    expect(bounds([])).toEqual({ minX: 0, minY: 0, maxX: 0, maxY: 0 })
    expect(bounds([{ id: 'a', x: 10, y: 20, label: 'A' }, { id: 'b', x: 30, y: 5, label: 'B' }])).toEqual({ minX: 10, minY: 5, maxX: 30, maxY: 20 })
  })
})
