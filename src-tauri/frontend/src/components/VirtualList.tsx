import { createSignal, createMemo, For, type JSX } from 'solid-js'

interface VirtualListProps<T> {
  items: T[]
  itemHeight: number
  containerHeight: number
  overscan?: number
  renderItem: (item: T, index: number) => JSX.Element
}

export function VirtualList<T>(props: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = createSignal(0)
  const overscan = () => props.overscan ?? 5

  const startIndex = createMemo(() =>
    Math.max(0, Math.floor(scrollTop() / props.itemHeight) - overscan())
  )

  const endIndex = createMemo(() =>
    Math.min(props.items.length, startIndex() + Math.ceil(props.containerHeight / props.itemHeight) + overscan() * 2)
  )

  const visibleItems = createMemo(() =>
    props.items.slice(startIndex(), endIndex()).map((item, i) => ({
      item, index: startIndex() + i
    }))
  )

  const totalHeight = createMemo(() => props.items.length * props.itemHeight)

  return (
    <div
      style={{ height: `${props.containerHeight}px`, overflow: 'auto' }}
      onScroll={(e) => setScrollTop((e.target as HTMLElement).scrollTop)}
    >
      <div style={{ height: `${totalHeight()}px`, position: 'relative' }}>
        <div style={{
          position: 'absolute',
          top: 0,
          transform: `translateY(${startIndex() * props.itemHeight}px)`
        }}>
          <For each={visibleItems()}>
            {(item) => props.renderItem(item.item, item.index)}
          </For>
        </div>
      </div>
    </div>
  )
}
