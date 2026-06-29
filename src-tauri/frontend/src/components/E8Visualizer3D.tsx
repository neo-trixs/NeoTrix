import { useRef, useMemo, useEffect } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import * as THREE from 'three';
import {
  generateE8Roots,
  rotateE8,
  projectTo3D,
  findNeighborEdges,
  E8Root,
} from '../engine/e8-roots';

interface E8AttentionData {
  scores: number[];
  top_roots: [string, number][];
  tick: number;
}

interface E8Visualizer3DProps {
  data?: E8AttentionData | null;
  cycle?: number;
}

const ROOT_COUNT = 240;
const SCALE = 5;

const BASIS: number[][] = [
  [1, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 0, 0, 0, 0, 0, 0],
  [0, 0, 1, 0, 0, 0, 0, 0],
];

const INT_RGB: [number, number, number] = [0, 1, 0.8];
const HALF_RGB: [number, number, number] = [1, 0.533, 0.267];

const scanState = { position: 0, time: 0 };

function createGlowTexture(): THREE.Texture {
  const canvas = document.createElement('canvas');
  canvas.width = 64;
  canvas.height = 64;
  const ctx = canvas.getContext('2d')!;
  const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
  g.addColorStop(0, 'rgba(255,255,255,1)');
  g.addColorStop(0.15, 'rgba(255,255,255,0.95)');
  g.addColorStop(0.5, 'rgba(255,255,255,0.4)');
  g.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, 64, 64);
  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  return texture;
}

function ScanningBeam() {
  const meshRef = useRef<THREE.Mesh>(null);

  const material = useMemo(() => new THREE.ShaderMaterial({
    uniforms: {
      uScan: { value: 0 },
      uTime: { value: 0 },
    },
    transparent: true,
    side: THREE.DoubleSide,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
    vertexShader: `
      varying vec3 vPos;
      void main() {
        vPos = position;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: `
      uniform float uScan;
      uniform float uTime;
      varying vec3 vPos;
      void main() {
        float bx = (uScan - 0.5) * 24.0;
        float dx = abs(vPos.x - bx);
        float a = exp(-dx * dx * 0.08) * 0.25;
        a *= (0.6 + 0.4 * sin(uTime * 3.0));
        gl_FragColor = vec4(0.15, 0.55, 1.0, a);
      }
    `,
  }), []);

  useFrame(() => {
    const s = scanState;
    material.uniforms.uScan.value = s.position;
    material.uniforms.uTime.value = s.time;
    const bx = (s.position - 0.5) * 24.0;
    meshRef.current?.position.set(bx, 0, 0);
  });

  return (
    <mesh ref={meshRef}>
      <planeGeometry args={[0.12, 14]} />
      <primitive object={material} attach="material" />
    </mesh>
  );
}

function E8Scene({ data }: { data?: E8AttentionData | null }) {
  const timeRef = useRef(0);

  const { rootData, edgePairs, staticVectors } = useMemo(() => {
    const roots = generateE8Roots();
    const vectors = roots.map(r => r.vector);
    const edges = findNeighborEdges(vectors);
    return { rootData: roots, edgePairs: edges, staticVectors: vectors };
  }, []);

  const glowTexture = useMemo(() => createGlowTexture(), []);

  const rootGeom = useMemo(() => {
    const positions = new Float32Array(ROOT_COUNT * 3);
    for (let i = 0; i < ROOT_COUNT; i++) {
      const [x, y, z] = projectTo3D(staticVectors[i], BASIS);
      positions[i * 3] = x * SCALE;
      positions[i * 3 + 1] = y * SCALE;
      positions[i * 3 + 2] = z * SCALE;
    }

    const colors = new Float32Array(ROOT_COUNT * 3);
    for (let i = 0; i < ROOT_COUNT; i++) {
      const base = rootData[i].isInteger ? INT_RGB : HALF_RGB;
      colors[i * 3] = base[0];
      colors[i * 3 + 1] = base[1];
      colors[i * 3 + 2] = base[2];
    }

    const geom = new THREE.BufferGeometry();
    geom.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    geom.setAttribute('color', new THREE.BufferAttribute(colors, 3));
    return geom;
  }, []);

  const edgeGeom = useMemo(() => {
    const posAttr = rootGeom.attributes.position;
    const src = posAttr.array as Float32Array;
    const positions = new Float32Array(edgePairs.length * 6);
    for (let e = 0; e < edgePairs.length; e++) {
      const [i, j] = edgePairs[e];
      positions[e * 6] = src[i * 3];
      positions[e * 6 + 1] = src[i * 3 + 1];
      positions[e * 6 + 2] = src[i * 3 + 2];
      positions[e * 6 + 3] = src[j * 3];
      positions[e * 6 + 4] = src[j * 3 + 1];
      positions[e * 6 + 5] = src[j * 3 + 2];
    }
    const geom = new THREE.BufferGeometry();
    geom.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    return geom;
  }, []);

  const rootMat = useMemo(() => new THREE.PointsMaterial({
    size: 0.55,
    map: glowTexture,
    vertexColors: true,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
    sizeAttenuation: true,
  }), [glowTexture]);

  const edgeMat = useMemo(() => new THREE.LineBasicMaterial({
    color: 0x4488ff,
    transparent: true,
    opacity: 0.12,
    depthWrite: false,
  }), []);

  useFrame((_, delta) => {
    timeRef.current += delta;
    const t = timeRef.current;

    scanState.time += delta;
    scanState.position = (Math.sin(scanState.time * 0.25) + 1) * 0.5;

    const rotated = rotateE8(staticVectors, t);
    const posArr = rootGeom.attributes.position.array as Float32Array;

    for (let i = 0; i < ROOT_COUNT; i++) {
      const [x, y, z] = projectTo3D(rotated[i], BASIS);
      posArr[i * 3] = x * SCALE;
      posArr[i * 3 + 1] = y * SCALE;
      posArr[i * 3 + 2] = z * SCALE;
    }
    rootGeom.attributes.position.needsUpdate = true;

    const edgeArr = edgeGeom.attributes.position.array as Float32Array;
    for (let e = 0; e < edgePairs.length; e++) {
      const [i, j] = edgePairs[e];
      edgeArr[e * 6] = posArr[i * 3];
      edgeArr[e * 6 + 1] = posArr[i * 3 + 1];
      edgeArr[e * 6 + 2] = posArr[i * 3 + 2];
      edgeArr[e * 6 + 3] = posArr[j * 3];
      edgeArr[e * 6 + 4] = posArr[j * 3 + 1];
      edgeArr[e * 6 + 5] = posArr[j * 3 + 2];
    }
    edgeGeom.attributes.position.needsUpdate = true;

    const scores = data?.scores;
    const colorArr = rootGeom.attributes.color.array as Float32Array;
    if (scores && scores.length === ROOT_COUNT) {
      let dirty = false;
      for (let i = 0; i < ROOT_COUNT; i++) {
        const s = Math.min(1, Math.max(0, scores[i] ?? 0));
        const base = rootData[i].isInteger ? INT_RGB : HALF_RGB;
        const b = 0.3 + 0.7 * s;
        const r = base[0] * b;
        const g = base[1] * b;
        const bl = base[2] * b;
        if (colorArr[i * 3] !== r || colorArr[i * 3 + 1] !== g || colorArr[i * 3 + 2] !== bl) {
          colorArr[i * 3] = r;
          colorArr[i * 3 + 1] = g;
          colorArr[i * 3 + 2] = bl;
          dirty = true;
        }
      }
      if (dirty) rootGeom.attributes.color.needsUpdate = true;
    } else {
      let dirty = false;
      for (let i = 0; i < ROOT_COUNT; i++) {
        const base = rootData[i].isInteger ? INT_RGB : HALF_RGB;
        if (colorArr[i * 3] !== base[0] || colorArr[i * 3 + 1] !== base[1] || colorArr[i * 3 + 2] !== base[2]) {
          colorArr[i * 3] = base[0];
          colorArr[i * 3 + 1] = base[1];
          colorArr[i * 3 + 2] = base[2];
          dirty = true;
        }
      }
      if (dirty) rootGeom.attributes.color.needsUpdate = true;
    }
  });

  return (
    <>
      <points geometry={rootGeom} material={rootMat} />
      <lineSegments geometry={edgeGeom} material={edgeMat} />
      <ScanningBeam />
      <OrbitControls
        enableDamping
        dampingFactor={0.08}
        autoRotate={false}
        minDistance={5}
        maxDistance={30}
        target={[0, 0, 0]}
      />
      <ambientLight intensity={0.3} />
      <directionalLight position={[8, 12, 8]} intensity={0.6} />
      <directionalLight position={[-8, 4, -8]} intensity={0.2} color="#4466ff" />
    </>
  );
}

export default function E8Visualizer3D({ data, cycle }: E8Visualizer3DProps) {
  useEffect(() => {
    return () => { scanState.position = 0; scanState.time = 0; };
  }, []);

  return (
    <div style={{
      width: '100%',
      height: 420,
      borderRadius: 8,
      overflow: 'hidden',
      background: '#0a0a14',
      position: 'relative',
    }}>
      <Canvas
        camera={{ position: [16, 12, 16], fov: 45, near: 0.1, far: 50 }}
        gl={{ antialias: true, alpha: true }}
        dpr={[1, 2]}
      >
        <E8Scene data={data} />
      </Canvas>
      {cycle !== undefined && (
        <div style={{
          position: 'absolute',
          bottom: 8,
          right: 12,
          fontSize: 10,
          opacity: 0.4,
          fontFamily: 'monospace',
          color: '#888',
        }}>
          cycle #{cycle}
        </div>
      )}
    </div>
  );
}
