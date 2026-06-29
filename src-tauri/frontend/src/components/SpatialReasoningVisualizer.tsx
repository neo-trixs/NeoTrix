import { useRef, useMemo, useEffect, useState } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Text } from '@react-three/drei';
import * as THREE from 'three';
import { forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide } from 'd3-force-3d';

export interface ConceptNode {
  id: string;
  label: string;
  x: number;
  z: number;
  confidence: number;
  color?: string;
}

export interface ReasoningStep {
  fromId: string;
  toId: string;
  score: number;
  label?: string;
}

export interface ReasoningPath {
  id: string;
  name: string;
  steps: ReasoningStep[];
  isActive?: boolean;
}

interface Props {
  concepts?: ConceptNode[];
  paths?: ReasoningPath[];
  width?: number;
  height?: number;
}

const TERRAIN_RES = 64;
const TERRAIN_SIZE = 24;

function genTerrain(concepts: ConceptNode[]) {
  const R = TERRAIN_RES;
  const positions = new Float32Array(R * R * 3);
  const colors = new Float32Array(R * R * 3);
  const S = TERRAIN_SIZE;

  for (let iz = 0; iz < R; iz++) {
    for (let ix = 0; ix < R; ix++) {
      const wx = (ix / R - 0.5) * S;
      const wz = (iz / R - 0.5) * S;
      let h = 0;
      for (const c of concepts) {
        const dx = wx - c.x, dz = wz - c.z;
        h += c.confidence * Math.exp(-(dx * dx + dz * dz) * 0.3);
      }
      h = Math.min(h, 2.0);
      const i = (iz * R + ix) * 3;
      positions[i] = wx; positions[i + 1] = h; positions[i + 2] = wz;
      const t = h / 2.0;
      const clr = new THREE.Color().setHSL(0.58 - t * 0.5, 0.55, 0.15 + t * 0.5);
      colors[i] = clr.r; colors[i + 1] = clr.g; colors[i + 2] = clr.b;
    }
  }

  const indices: number[] = [];
  for (let iz = 0; iz < R - 1; iz++) {
    for (let ix = 0; ix < R - 1; ix++) {
      const a = iz * R + ix, b = a + 1;
      const c = (iz + 1) * R + ix, d = c + 1;
      indices.push(a, b, c, b, d, c);
    }
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  geo.setIndex(indices);
  geo.computeVertexNormals();
  return geo;
}

function getPathPoints(
  path: ReasoningPath,
  positionsRef: React.MutableRefObject<Map<string, THREE.Vector3>>,
  concepts: Map<string, ConceptNode>
): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  for (const s of path.steps) {
    const f = concepts.get(s.fromId);
    const t = concepts.get(s.toId);
    if (!f || !t) continue;
    const fp = positionsRef.current.get(s.fromId);
    const tp = positionsRef.current.get(s.toId);
    if (!fp || !tp) continue;
    pts.push(new THREE.Vector3(fp.x, fp.y + 0.1, fp.z));
    pts.push(new THREE.Vector3(
      (fp.x + tp.x) / 2,
      Math.max(fp.y, tp.y) + ((f.confidence + t.confidence) / 2) * 0.8 + 0.4,
      (fp.z + tp.z) / 2
    ));
    pts.push(new THREE.Vector3(tp.x, tp.y + 0.1, tp.z));
  }
  return pts;
}

function buildCurveGeometry(points: THREE.Vector3[], isActive: boolean) {
  if (points.length < 2) return null;
  const curve = new THREE.CatmullRomCurve3(points);
  return new THREE.TubeGeometry(curve, 64, isActive ? 0.08 : 0.04, 8, false);
}

function ConceptSphere({
  concept, active, positionsRef, hoveredId, onHover
}: {
  concept: ConceptNode;
  active: boolean;
  positionsRef: React.MutableRefObject<Map<string, THREE.Vector3>>;
  hoveredId: string | null;
  onHover: (id: string | null) => void;
}) {
  const groupRef = useRef<THREE.Group>(null);
  const r = 0.12 + concept.confidence * 0.28;
  const clr = concept.color || '#4488ff';
  const isHovered = hoveredId === concept.id;

  useFrame(() => {
    const pos = positionsRef.current.get(concept.id);
    if (pos && groupRef.current) {
      groupRef.current.position.set(pos.x, pos.y + 0.1, pos.z);
    }
  });

  return (
    <group ref={groupRef} position={[concept.x, concept.confidence * 1.2 + 0.1, concept.z]}>
      <mesh
        onPointerEnter={() => onHover(concept.id)}
        onPointerLeave={() => onHover(null)}
      >
        <sphereGeometry args={[r, 16, 16]} />
        <meshPhongMaterial
          color={clr}
          emissive={clr}
          emissiveIntensity={isHovered || active ? 0.5 : 0.08}
        />
      </mesh>
      <Text
        position={[0, r + 0.18, 0]}
        fontSize={0.22}
        color={isHovered || active ? '#ffffff' : '#aaa'}
        anchorX="center"
        anchorY="bottom"
        maxWidth={1.8}
      >
        {concept.label}
      </Text>
    </group>
  );
}

function ReasonCurve({
  path, positionsRef, concepts, hoveredPathIds
}: {
  path: ReasoningPath;
  positionsRef: React.MutableRefObject<Map<string, THREE.Vector3>>;
  concepts: Map<string, ConceptNode>;
  hoveredPathIds: Set<string>;
}) {
  const meshRef = useRef<THREE.Mesh>(null);
  const lastKeyRef = useRef<string>('');
  const highlighted = hoveredPathIds.has(path.id) || (path.isActive || false);

  useFrame(() => {
    const pts = getPathPoints(path, positionsRef, concepts);
    if (pts.length < 2) return;

    const key = pts.map(p => `${p.x.toFixed(2)},${p.y.toFixed(2)},${p.z.toFixed(2)}`).join('|');
    if (key === lastKeyRef.current) return;
    lastKeyRef.current = key;

    if (meshRef.current) {
      meshRef.current.geometry.dispose();
    }

    const geo = buildCurveGeometry(pts, highlighted);
    if (geo && meshRef.current) {
      meshRef.current.geometry = geo;
    }
  });

  return (
    <mesh ref={meshRef}>
      <meshPhongMaterial
        color={highlighted ? '#00ffcc' : '#4466aa'}
        transparent
        opacity={highlighted ? 0.85 : 0.25}
        emissive={highlighted ? '#00ffcc' : '#000000'}
        emissiveIntensity={highlighted ? 0.3 : 0}
      />
    </mesh>
  );
}

function FlowParticles({
  path, positionsRef, concepts, hoveredPathIds
}: {
  path: ReasoningPath;
  positionsRef: React.MutableRefObject<Map<string, THREE.Vector3>>;
  concepts: Map<string, ConceptNode>;
  hoveredPathIds: Set<string>;
}) {
  const meshRef = useRef<THREE.Points>(null);
  const curveRef = useRef<THREE.CatmullRomCurve3 | null>(null);
  const lastKeyRef = useRef<string>('');
  const count = 60;

  const geo = useMemo(() => {
    const g = new THREE.BufferGeometry();
    const pos = new Float32Array(count * 3);
    g.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    return g;
  }, []);

  const highlighted = hoveredPathIds.has(path.id) || (path.isActive || false);

  useFrame(() => {
    if (!meshRef.current) return;

    const pts = getPathPoints(path, positionsRef, concepts);
    const key = pts.map(p => `${p.x.toFixed(1)},${p.y.toFixed(1)},${p.z.toFixed(1)}`).join('|');
    if (key !== lastKeyRef.current && pts.length >= 2) {
      curveRef.current = new THREE.CatmullRomCurve3(pts);
      lastKeyRef.current = key;
    }

    const curve = curveRef.current;
    if (!curve) return;

    const pos = meshRef.current.geometry.attributes.position.array as Float32Array;
    const avgScore = path.steps.reduce((sum, s) => sum + s.score, 0) / path.steps.length;
    const speed = 0.08 + avgScore * 0.12;

    for (let i = 0; i < count; i++) {
      const t = (i / count + performance.now() * 0.001 * speed) % 1;
      const p = curve.getPoint(t);
      pos[i * 3] = p.x;
      pos[i * 3 + 1] = p.y;
      pos[i * 3 + 2] = p.z;
    }
    meshRef.current.geometry.attributes.position.needsUpdate = true;
  });

  return (
    <points ref={meshRef} geometry={geo}>
      <pointsMaterial
        size={0.1}
        color={highlighted ? '#00ffcc' : '#4488ff'}
        transparent
        opacity={highlighted ? 0.9 : 0.3}
        blending={THREE.AdditiveBlending}
        depthWrite={false}
      />
    </points>
  );
}

function Scene({ concepts, paths }: { concepts: ConceptNode[]; paths: ReasoningPath[] }) {
  const cmap = useMemo(() => new Map(concepts.map(c => [c.id, c])), [concepts]);
  const positionsRef = useRef<Map<string, THREE.Vector3>>(new Map());
  const simRef = useRef<any>(null);
  const simNodesRef = useRef<any[]>([]);
  const [hoveredId, setHoveredId] = useState<string | null>(null);

  useEffect(() => {
    simRef.current?.stop();

    const simNodes = concepts.map(c => ({
      id: c.id,
      x: c.x || (Math.random() - 0.5) * 6,
      y: c.confidence * 1.2,
      z: c.z || (Math.random() - 0.5) * 6,
    }));
    simNodesRef.current = simNodes;

    for (const n of simNodes) {
      positionsRef.current.set(n.id, new THREE.Vector3(n.x, n.y, n.z));
    }

    const simLinks: any[] = [];
    for (const p of paths) {
      for (const s of p.steps) {
        simLinks.push({ source: s.fromId, target: s.toId, score: s.score });
      }
    }

    const simulation = forceSimulation(simNodes)
      .force('link', forceLink(simLinks).id((d: any) => d.id).distance(50))
      .force('charge', forceManyBody().strength(-80))
      .force('center', forceCenter(0, 0, 0))
      .force('collide', forceCollide().radius(12))
      .alphaDecay(0.02);

    simulation.stop();
    simulation.alpha(1);
    simRef.current = simulation;

    return () => {
      simulation.stop();
    };
  }, [concepts, paths]);

  useFrame(() => {
    if (!simRef.current) return;
    simRef.current.tick();
    const nodes = simNodesRef.current;
    for (const n of nodes) {
      const pos = positionsRef.current.get(n.id);
      if (pos) {
        pos.set(n.x, n.y, n.z);
      }
    }
  });

  const terrain = useMemo(() => genTerrain(concepts), [concepts]);

  const activeIds = useMemo(() => {
    const s = new Set<string>();
    for (const p of paths) {
      if (!p.isActive) continue;
      for (const st of p.steps) { s.add(st.fromId); s.add(st.toId); }
    }
    return s;
  }, [paths]);

  const hoveredPathIds = useMemo(() => {
    const s = new Set<string>();
    if (!hoveredId) return s;
    for (const p of paths) {
      for (const st of p.steps) {
        if (st.fromId === hoveredId || st.toId === hoveredId) {
          s.add(p.id);
          break;
        }
      }
    }
    return s;
  }, [paths, hoveredId]);

  return (
    <>
      <mesh geometry={terrain} position={[0, -0.3, 0]}>
        <meshPhongMaterial vertexColors />
      </mesh>

      {paths.map(p => (
        <ReasonCurve key={p.id} path={p} positionsRef={positionsRef} concepts={cmap} hoveredPathIds={hoveredPathIds} />
      ))}
      {paths.filter(p => p.isActive).map(p => (
        <FlowParticles key={`flow-${p.id}`} path={p} positionsRef={positionsRef} concepts={cmap} hoveredPathIds={hoveredPathIds} />
      ))}

      {concepts.map(c => (
        <ConceptSphere
          key={c.id}
          concept={c}
          active={activeIds.has(c.id)}
          positionsRef={positionsRef}
          hoveredId={hoveredId}
          onHover={setHoveredId}
        />
      ))}

      <OrbitControls enableDamping dampingFactor={0.08} minDistance={3} maxDistance={30} target={[0, 0.5, 0]} />
      <ambientLight intensity={0.35} />
      <directionalLight position={[5, 10, 5]} intensity={0.6} />
      <directionalLight position={[-5, 5, -5]} intensity={0.2} color="#4466ff" />
      <gridHelper args={[20, 20, '#333', '#222']} position={[0, -0.35, 0]} />
    </>
  );
}

export function demoConcepts(): ConceptNode[] {
  return [
    { id: 'c1', label: 'VSA Core', x: -3, z: 2, confidence: 0.95, color: '#00ffcc' },
    { id: 'c2', label: 'E8 Lattice', x: 4, z: -1, confidence: 0.88, color: '#4488ff' },
    { id: 'c3', label: 'HyperCube KB', x: -1, z: -4, confidence: 0.72, color: '#ff8844' },
    { id: 'c4', label: 'Attention Flow', x: 5, z: 3, confidence: 0.65, color: '#ff44aa' },
    { id: 'c5', label: 'Memory Recall', x: -4, z: -2, confidence: 0.81, color: '#44ff88' },
    { id: 'c6', label: 'GWT Workspace', x: 1, z: 5, confidence: 0.59, color: '#aa88ff' },
    { id: 'c7', label: 'Self Model', x: -2, z: -5, confidence: 0.45, color: '#ff6688' },
    { id: 'c8', label: 'Emotion State', x: 6, z: -3, confidence: 0.38, color: '#ffaa44' },
    { id: 'c9', label: 'Reasoning Chain', x: -6, z: 4, confidence: 0.92, color: '#00ff88' },
    { id: 'c10', label: 'Goal Stack', x: 0, z: 0, confidence: 0.70, color: '#88aaff' },
  ];
}

export function demoPaths(): ReasoningPath[] {
  return [
    {
      id: 'p1', name: 'Primary reasoning', isActive: true,
      steps: [
        { fromId: 'c1', toId: 'c10', score: 0.9 },
        { fromId: 'c10', toId: 'c2', score: 0.8 },
        { fromId: 'c2', toId: 'c4', score: 0.7 },
        { fromId: 'c4', toId: 'c6', score: 0.6 },
      ],
    },
    {
      id: 'p2', name: 'Memory lookup', isActive: false,
      steps: [
        { fromId: 'c3', toId: 'c5', score: 0.7 },
        { fromId: 'c5', toId: 'c7', score: 0.5 },
      ],
    },
    {
      id: 'p3', name: 'Emotion appraisal', isActive: false,
      steps: [
        { fromId: 'c6', toId: 'c8', score: 0.6 },
        { fromId: 'c8', toId: 'c7', score: 0.4 },
      ],
    },
  ];
}

export default function SpatialReasoningVisualizer({
  concepts = demoConcepts(),
  paths = demoPaths(),
  width = 600,
  height = 400,
}: Props) {
  return (
    <div style={{ width, height, borderRadius: 8, overflow: 'hidden', background: '#0a0a14' }}>
      <Canvas camera={{ position: [12, 8, 12], fov: 45, near: 0.1, far: 50 }} gl={{ antialias: true, alpha: true }} dpr={[1, 2]}>
        <Scene concepts={concepts} paths={paths} />
      </Canvas>
    </div>
  );
}
