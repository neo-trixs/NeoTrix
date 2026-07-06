import React, { useCallback, useEffect, useMemo, useState } from "react";
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  MarkerType,
  type Node,
  type Edge,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import dagre from "@dagrejs/dagre";
import { kbGetRelated, kbGetNode } from "../lib/api";
import type { KbSearchResult, KbNode } from "../lib/api";

interface KbNodeRaw {
  id: string;
  node_type: string;
  title: string;
  summary: string | null;
  domain: string | null;
  confidence: number;
  importance: number;
}

interface KbEdge {
  id: string;
  source_id: string;
  target_id: string;
  relation_type: string;
  weight: number;
}

interface KbGraphResponse {
  nodes: KbNodeRaw[];
  edges: KbEdge[];
}

const NODE_COLORS: Record<string, string> = {
  Repository: "#E85454",
  Concept: "#4A90D9",
  Paper: "#7B68EE",
  Insight: "#F5A623",
  Resource: "#50C878",
  Article: "#FF6B6B",
  CodeSnippet: "#00D68F",
  Tool: "#FFA94D",
  Person: "#A29BFE",
  default: "#888",
};

const TYPE_OPTIONS = [
  "All",
  "Repository",
  "Concept",
  "Paper",
  "Insight",
  "Resource",
  "Article",
  "CodeSnippet",
  "Tool",
];

function layoutHierarchy(nodes: Node[], edges: Edge[]): Node[] {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: "LR", nodesep: 80, ranksep: 150 });
  nodes.forEach((n) => g.setNode(n.id, { width: 180, height: 50 }));
  edges.forEach((e) => g.setEdge(e.source, e.target));
  dagre.layout(g);
  return nodes.map((n) => {
    const pos = g.node(n.id);
    if (pos) {
      return { ...n, position: { x: pos.x - 90, y: pos.y - 25 } };
    }
    return n;
  });
}

function layoutGrid(nodes: Node[]): Node[] {
  const SPACING_X = 250;
  const SPACING_Y = 80;
  const nodesPerRow = Math.ceil(Math.sqrt(nodes.length));
  return nodes.map((n, i) => {
    const col = i % nodesPerRow;
    const row = Math.floor(i / nodesPerRow);
    return {
      ...n,
      position: { x: col * SPACING_X + 40, y: row * SPACING_Y + 40 },
    };
  });
}

function NodeDetailPanel({
  rawNode,
  onClose,
}: {
  rawNode: KbNodeRaw;
  onClose: () => void;
}) {
  const [detail, setDetail] = useState<KbNode | null>(null);
  const [related, setRelated] = useState<KbSearchResult[]>([]);
  const [expanded, setExpanded] = useState(false);
  const color = NODE_COLORS[rawNode.node_type] || NODE_COLORS.default;

  useEffect(() => {
    (async () => {
      try {
        const node = await kbGetNode(rawNode.id);
        if (node) setDetail(node);
      } catch {
        /* backend fallback */
      }
      try {
        const rel = await kbGetRelated(rawNode.id);
        setRelated(rel);
      } catch {
        /* backend fallback */
      }
    })();
  }, [rawNode.id]);

  const displaySummary =
    detail?.summary || rawNode.summary || "";

  return (
    <div
      style={{
        width: 340,
        flexShrink: 0,
        borderLeft:
          "1px solid var(--nt-border, rgba(255,255,255,0.06))",
        background: "var(--nt-glass-L1-bg, #16161E)",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "14px 16px",
          borderBottom:
            "1px solid var(--nt-border, rgba(255,255,255,0.06))",
        }}
      >
        <span
          style={{
            fontSize: 13,
            fontWeight: 600,
            color: "var(--nt-text, #EDEDED)",
          }}
        >
          Node Details
        </span>
        <button
          onClick={onClose}
          style={{
            background: "none",
            border: "none",
            color: "var(--nt-text-secondary, #808088)",
            cursor: "pointer",
            padding: 4,
            borderRadius: 4,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <svg
            viewBox="0 0 14 14"
            width={14}
            height={14}
            fill="none"
            stroke="currentColor"
            strokeWidth="1.3"
            strokeLinecap="round"
          >
            <path d="M3 3l8 8M11 3l-8 8" />
          </svg>
        </button>
      </div>

      <div
        style={{
          flex: 1,
          overflowY: "auto",
          padding: 16,
          display: "flex",
          flexDirection: "column",
          gap: 14,
        }}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <span
              style={{
                display: "inline-block",
                padding: "2px 8px",
                borderRadius: 10,
                fontSize: 10,
                fontWeight: 600,
                color: "#fff",
                background: color,
                letterSpacing: 0.3,
                textTransform: "uppercase",
              }}
            >
              {rawNode.node_type}
            </span>
            {rawNode.domain && (
              <span
                style={{
                  fontSize: 11,
                  color: "var(--nt-text-muted, #505058)",
                }}
              >
                {rawNode.domain.replace(/^https?:\/\//, "").split("/")[0]}
              </span>
            )}
          </div>
          <span
            style={{
              fontSize: 15,
              fontWeight: 600,
              color: "var(--nt-text, #EDEDED)",
              lineHeight: 1.3,
            }}
          >
            {detail?.title || rawNode.title}
          </span>
        </div>

        {displaySummary && (
          <div
            style={{
              fontSize: 13,
              lineHeight: 1.6,
              color: "var(--nt-text-secondary, #808088)",
            }}
          >
            <span>
              {expanded
                ? displaySummary
                : displaySummary.slice(0, 280)}
            </span>
            {displaySummary.length > 280 && (
              <button
                onClick={() => setExpanded(!expanded)}
                style={{
                  display: "block",
                  marginTop: 6,
                  background: "none",
                  border: "none",
                  color: "var(--nt-primary, #FF6B6B)",
                  cursor: "pointer",
                  fontSize: 12,
                  padding: 0,
                  fontWeight: 500,
                }}
              >
                {expanded ? "Show less" : "Show more"}
              </button>
            )}
          </div>
        )}

        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 8,
            padding: "10px 12px",
            borderRadius: 8,
            background: "var(--nt-glass-L2-bg, #181820)",
            border:
              "1px solid var(--nt-border, rgba(255,255,255,0.06))",
          }}
        >
          <MetaRow
            label="Confidence"
            value={(detail?.confidence ?? rawNode.confidence).toFixed(2)}
            bar={detail?.confidence ?? rawNode.confidence}
            color="var(--nt-primary, #FF6B6B)"
          />
          <MetaRow
            label="Importance"
            value={(detail?.importance ?? rawNode.importance).toFixed(2)}
            bar={detail?.importance ?? rawNode.importance}
            color="var(--nt-yellow, #FFD54F)"
          />
        </div>

        {related.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            <span
              style={{
                fontSize: 11,
                fontWeight: 600,
                color: "var(--nt-text-secondary, #808088)",
                textTransform: "uppercase",
                letterSpacing: 0.5,
              }}
            >
              Related ({related.length})
            </span>
            {related.map((r) => (
              <div
                key={r.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "8px 10px",
                  borderRadius: 6,
                  background: "var(--nt-glass-L2-bg, #181820)",
                  border:
                    "1px solid var(--nt-border, rgba(255,255,255,0.04))",
                  cursor: "pointer",
                }}
              >
                <div
                  style={{
                    width: 6,
                    height: 6,
                    borderRadius: "50%",
                    background:
                      NODE_COLORS[r.node_type] || NODE_COLORS.default,
                    flexShrink: 0,
                  }}
                />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: 12,
                      fontWeight: 500,
                      color: "var(--nt-text, #EDEDED)",
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {r.title}
                  </div>
                  <div
                    style={{
                      fontSize: 10,
                      color: "var(--nt-text-muted, #505058)",
                    }}
                  >
                    {r.node_type}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function MetaRow({
  label,
  value,
  bar,
  color,
}: {
  label: string;
  value: string;
  bar?: number;
  color?: string;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: 11,
          color: "var(--nt-text-muted, #505058)",
        }}
      >
        <span>{label}</span>
        <span style={{ color: "var(--nt-text-secondary, #808088)" }}>
          {value}
        </span>
      </div>
      {bar !== undefined && (
        <div
          style={{
            height: 3,
            borderRadius: 2,
            background: "var(--nt-gray-300, #404048)",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              width: `${Math.round(bar * 100)}%`,
              height: "100%",
              borderRadius: 2,
              background: color || "var(--nt-primary, #FF6B6B)",
              transition: "width 0.3s ease",
            }}
          />
        </div>
      )}
    </div>
  );
}

const KnowledgeGraphPage: React.FC = () => {
  const navigate = useNavigate();
  const [rawNodes, setRawNodes] = useState<KbNodeRaw[]>([]);
  const [rawEdges, setRawEdges] = useState<KbEdge[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterType, setFilterType] = useState("All");
  const [searchText, setSearchText] = useState("");
  const [stats, setStats] = useState<{
    total_nodes: number;
    total_edges: number;
  } | null>(null);
  const [layoutMode, setLayoutMode] = useState<"grid" | "hierarchy">(
    "grid"
  );
  const [selectedNode, setSelectedNode] = useState<KbNodeRaw | null>(
    null
  );

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  useEffect(() => {
    (async () => {
      try {
        const data = await invoke<KbGraphResponse>(
          "get_knowledge_graph"
        );
        setRawNodes(data.nodes);
        setRawEdges(data.edges);
        const s = await invoke<{
          total_nodes: number;
          total_edges: number;
        }>("get_knowledge_stats");
        setStats(s);
      } catch (e) {
        console.error("Failed to load knowledge graph:", e);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const filteredNodes = useMemo(() => {
    return rawNodes.filter((n) => {
      if (filterType !== "All" && n.node_type !== filterType)
        return false;
      if (searchText) {
        const q = searchText.toLowerCase();
        return (
          n.title.toLowerCase().includes(q) ||
          (n.summary && n.summary.toLowerCase().includes(q))
        );
      }
      return true;
    });
  }, [rawNodes, filterType, searchText]);

  const filteredNodeIds = useMemo(
    () => new Set(filteredNodes.map((n) => n.id)),
    [filteredNodes]
  );

  const filteredEdges = useMemo(
    () =>
      rawEdges.filter(
        (e) =>
          filteredNodeIds.has(e.source_id) &&
          filteredNodeIds.has(e.target_id)
      ),
    [rawEdges, filteredNodeIds]
  );

  useEffect(() => {
    const rfNodes: Node[] = filteredNodes.map((n) => ({
      id: n.id,
      type: "default",
      position: { x: 0, y: 0 },
      data: {
        label:
          n.title.length > 30
            ? n.title.slice(0, 30) + "..."
            : n.title,
        type: n.node_type,
        summary: n.summary || "",
        confidence: n.confidence,
      },
      style: {
        background:
          NODE_COLORS[n.node_type] || NODE_COLORS.default,
        color: "#fff",
        border: "none",
        borderRadius: 8,
        padding: "8px 14px",
        fontSize: 12,
        fontWeight: 600,
        boxShadow: "0 2px 8px rgba(0,0,0,0.25)",
        width: 180,
      },
    }));

    const rfEdges: Edge[] = filteredEdges.map((e) => ({
      id: e.id,
      source: e.source_id,
      target: e.target_id,
      label: e.relation_type,
      style: {
        stroke: "#666",
        strokeWidth: Math.max(1, e.weight),
      },
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: "#666",
      },
      labelStyle: { fontSize: 10, fill: "#999" },
    }));

    const positioned =
      layoutMode === "hierarchy"
        ? layoutHierarchy(rfNodes, rfEdges)
        : layoutGrid(rfNodes);

    setNodes(positioned);
    setEdges(rfEdges);
  }, [filteredNodes, filteredEdges, layoutMode, setNodes, setEdges]);

  const onNodeClick = useCallback(
    (_: React.MouseEvent, node: Node) => {
      const kbNode = rawNodes.find((n) => n.id === node.id);
      if (kbNode) {
        setSelectedNode(kbNode);
      }
    },
    [rawNodes]
  );

  const handleExportSvg = useCallback(() => {
    const el = document.querySelector(".react-flow__viewport");
    if (el) {
      const svgClone = el.cloneNode(true) as HTMLElement;
      const serializer = new XMLSerializer();
      const svgStr = serializer.serializeToString(svgClone);
      const blob = new Blob([svgStr], { type: "image/svg+xml" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "knowledge-graph.svg";
      a.click();
      URL.revokeObjectURL(url);
    }
  }, []);

  if (loading) {
    return (
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "var(--nt-text-secondary, #808088)",
          fontFamily: "var(--nt-font-family, system-ui)",
        }}
      >
        Loading knowledge graph...
      </div>
    );
  }

  return (
    <div
      style={{
        display: "flex",
        height: "100%",
        background: "var(--nt-bg, #181820)",
        color: "var(--nt-text, #EDEDED)",
        fontFamily: "var(--nt-font-family, system-ui)",
      }}
    >
      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          minWidth: 0,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            padding: "12px 20px",
            borderBottom:
              "1px solid var(--nt-border, rgba(255,255,255,0.06))",
            background: "var(--nt-glass-L1-bg, #16161E)",
            flexWrap: "wrap",
          }}
        >
          <span style={{ fontWeight: 600, fontSize: 14 }}>
            {stats
              ? `${stats.total_nodes} nodes · ${stats.total_edges} edges`
              : "Knowledge Graph"}
          </span>

          <div style={{ flex: 1 }} />

          <button
            onClick={() => navigate("/explore")}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 4,
              padding: "5px 10px",
              borderRadius: 6,
              border:
                "1px solid var(--nt-border, rgba(255,255,255,0.06))",
              background: "transparent",
              color: "var(--nt-text-secondary, #808088)",
              cursor: "pointer",
              fontSize: 12,
            }}
          >
            <svg
              viewBox="0 0 12 12"
              width={12}
              height={12}
              fill="none"
              stroke="currentColor"
              strokeWidth="1.3"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M2.5 6h7M6 2.5L2.5 6 6 9.5" />
            </svg>
            Feed
          </button>

          <button
            onClick={() =>
              setLayoutMode(
                layoutMode === "grid" ? "hierarchy" : "grid"
              )
            }
            style={{
              display: "flex",
              alignItems: "center",
              gap: 4,
              padding: "5px 10px",
              borderRadius: 6,
              border: "1px solid var(--nt-primary, #FF6B6B)",
              background:
                "var(--nt-primary-light-bg, rgba(232,84,84,0.10))",
              color: "var(--nt-primary, #FF6B6B)",
              cursor: "pointer",
              fontSize: 12,
              fontWeight: 500,
            }}
          >
            {layoutMode === "grid" ? "Grid" : "Hierarchy"}
          </button>

          <button
            onClick={handleExportSvg}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 4,
              padding: "5px 10px",
              borderRadius: 6,
              border:
                "1px solid var(--nt-border, rgba(255,255,255,0.06))",
              background: "transparent",
              color: "var(--nt-text-secondary, #808088)",
              cursor: "pointer",
              fontSize: 12,
            }}
          >
            <svg
              viewBox="0 0 12 12"
              width={12}
              height={12}
              fill="none"
              stroke="currentColor"
              strokeWidth="1.3"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M10 7.5v2a.5.5 0 01-.5.5h-7a.5.5 0 01-.5-.5v-2" />
              <path d="M6 7V2M4 4l2 3 2-3" />
            </svg>
            Export SVG
          </button>

          <input
            type="text"
            placeholder="Search nodes..."
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            style={{
              padding: "5px 10px",
              borderRadius: 6,
              border:
                "1px solid var(--nt-border, rgba(255,255,255,0.06))",
              background: "var(--nt-glass-L2-bg, #181820)",
              color: "var(--nt-text, #EDEDED)",
              fontSize: 12,
              width: 160,
              outline: "none",
            }}
          />

          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            style={{
              padding: "5px 8px",
              borderRadius: 6,
              border:
                "1px solid var(--nt-border, rgba(255,255,255,0.06))",
              background: "var(--nt-glass-L2-bg, #181820)",
              color: "var(--nt-text, #EDEDED)",
              fontSize: 12,
              outline: "none",
            }}
          >
            {TYPE_OPTIONS.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>

          <span
            style={{
              fontSize: 12,
              color: "var(--nt-text-secondary, #808088)",
            }}
          >
            Showing {filteredNodes.length} nodes
          </span>
        </div>

        <div style={{ flex: 1 }}>
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onNodeClick={onNodeClick}
            fitView
            attributionPosition="bottom-left"
            minZoom={0.1}
            maxZoom={3}
          >
            <Background color="#444" gap={20} />
            <Controls
              style={{
                borderRadius: 8,
                background: "var(--nt-glass-L2-bg, #181820)",
                border:
                  "1px solid var(--nt-border, rgba(255,255,255,0.06))",
              }}
            />
            <MiniMap
              style={{
                background: "var(--nt-glass-L2-bg, #181820)",
                border:
                  "1px solid var(--nt-border, rgba(255,255,255,0.06))",
                borderRadius: 8,
              }}
              nodeColor={
                (n) =>
                  (n.style?.background as string) || "#888"
              }
              nodeStrokeWidth={0}
              maskColor="rgba(0,0,0,0.4)"
            />
          </ReactFlow>
        </div>
      </div>

      {selectedNode && (
        <NodeDetailPanel
          rawNode={selectedNode}
          onClose={() => setSelectedNode(null)}
        />
      )}
    </div>
  );
};

export default KnowledgeGraphPage;
