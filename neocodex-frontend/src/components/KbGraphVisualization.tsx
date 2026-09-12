import { createSignal, onMount, For, Show } from 'solid-js'
import { kb } from '../api/domain'

interface KbNode {
  id: string
  name: string
  type: string
}

interface KbEdge {
  source: string
  target: string
  relation: string
}

export function KbGraphVisualization() {
  const [nodes, setNodes] = createSignal<KbNode[]>([])
  const [edges, setEdges] = createSignal<KbEdge[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [selectedNode, setSelectedNode] = createSignal<KbNode | null>(null)

  const fetchGraph = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await kb.graph()
      setNodes(result.nodes || [])
      setEdges(result.edges || [])
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(fetchGraph)

  const handleNodeClick = (node: KbNode) => {
    setSelectedNode(node)
  }

  return (
    <div class="kb-graph-visualization">
      <div class="header">
        <h3>知识图谱</h3>
        <button onClick={fetchGraph} disabled={loading()}>
          {loading() ? '加载中...' : '刷新'}
        </button>
      </div>
      
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
      
      <div class="graph-container">
        <svg width="100%" height="400">
          <For each={edges()}>
            {(edge) => {
              const sourceNode = nodes().find(n => n.id === edge.source)
              const targetNode = nodes().find(n => n.id === edge.target)
              if (!sourceNode || !targetNode) return null
              return (
                <line
                  x1={100 + nodes().indexOf(sourceNode) * 50}
                  y1={200}
                  x2={100 + nodes().indexOf(targetNode) * 50}
                  y2={200}
                  stroke="#666"
                  stroke-width="2"
                />
              )
            }}
          </For>
          <For each={nodes()}>
            {(node, i) => (
              <g
                transform={`translate(${100 + i() * 50}, 200)`}
                onClick={() => handleNodeClick(node)}
                style={{ cursor: 'pointer' }}
              >
                <circle r="20" fill={selectedNode()?.id === node.id ? '#3b82f6' : '#6b7280'} />
                <text
                  text-anchor="middle"
                  dy=".3em"
                  fill="white"
                  font-size="10"
                >
                  {node.name.substring(0, 5)}
                </text>
              </g>
            )}
          </For>
        </svg>
      </div>
      
      <Show when={selectedNode()}>
        <div class="node-detail">
          <h4>{selectedNode()!.name}</h4>
          <p>类型: {selectedNode()!.type}</p>
          <p>ID: {selectedNode()!.id}</p>
        </div>
      </Show>
    </div>
  )
}
