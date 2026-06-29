import { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { EffectComposer, Bloom } from '@react-three/postprocessing';
import * as THREE from 'three';

const TERRAIN_SIZE = 16;
const SEGMENTS = 64;

function getHeight(x: number, z: number): number {
  return (
    Math.sin(x * 0.3) * Math.cos(z * 0.4) * 1.2 +
    Math.sin(x * 0.7 + z * 0.5) * 0.8 +
    Math.cos(x * 0.15 + z * 0.25) * 1.5
  );
}

const WAYPOINTS_2D: [number, number][] = [
  [-6, -4], [-4, -5], [-2, -3], [0, -4], [2, -2],
  [4, -3], [5, 0], [3, 2], [1, 3], [-1, 4],
  [-3, 3], [-5, 2], [-6, 0], [-4, -2],
];

const TRAIL_COUNT = 5;

function Terrain() {
  const geometry = useMemo(() => {
    const geo = new THREE.PlaneGeometry(TERRAIN_SIZE, TERRAIN_SIZE, SEGMENTS, SEGMENTS);
    geo.rotateX(-Math.PI / 2);
    const pos = geo.attributes.position;
    const colors = new Float32Array(pos.count * 3);
    const low = new THREE.Color('#0a0a33');
    const mid = new THREE.Color('#4422aa');
    const high = new THREE.Color('#00ccaa');

    for (let i = 0; i < pos.count; i++) {
      const x = pos.getX(i);
      const z = pos.getZ(i);
      const h = getHeight(x, z);
      pos.setY(i, h);
      const t = (h + 3.5) / 7;
      const c = t < 0.5 ? low.clone().lerp(mid, t * 2) : mid.clone().lerp(high, (t - 0.5) * 2);
      colors[i * 3] = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;
    }
    geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
    geo.computeVertexNormals();
    return geo;
  }, []);

  return (
    <mesh geometry={geometry}>
      <meshPhongMaterial vertexColors side={THREE.DoubleSide} shininess={5} />
    </mesh>
  );
}

function TrajectoryPath() {
  const points = useMemo(() => {
    return WAYPOINTS_2D.map(([x, z]) => {
      const y = getHeight(x, z) + 0.3;
      return new THREE.Vector3(x, y, z);
    });
  }, []);

  const curve = useMemo(() => new THREE.CatmullRomCurve3(points), [points]);
  const tube = useMemo(() => {
    const geo = new THREE.TubeGeometry(curve, 64, 0.08, 8, false);
    return geo;
  }, [curve]);

  return (
    <mesh geometry={tube}>
      <meshBasicMaterial color="#00ffcc" transparent opacity={0.6} />
    </mesh>
  );
}

function ThinkingParticle() {
  const points = useMemo(() => {
    return WAYPOINTS_2D.map(([x, z]) => {
      const y = getHeight(x, z) + 0.3;
      return new THREE.Vector3(x, y, z);
    });
  }, []);

  const curve = useMemo(() => new THREE.CatmullRomCurve3(points), [points]);
  const progressRef = useRef(0);
  const trailRef = useRef<THREE.InstancedMesh>(null);
  const dummy = useMemo(() => new THREE.Object3D(), []);

  const trailPositions = useMemo(() => {
    const arr: number[] = [];
    for (let i = 0; i < TRAIL_COUNT; i++) arr.push(0);
    return arr;
  }, []);

  useFrame((_, delta) => {
    progressRef.current += delta * 0.08;
    if (progressRef.current > 1) progressRef.current -= 1;
    const t = progressRef.current;

    const speedMod = 0.6 + 0.4 * Math.sin(t * Math.PI * 6);
    const p = curve.getPoint(t);
    (trailRef.current?.parent as any)?.userData?.particlePos?.copy(p);
  });

  return <group />;
}

function TrailSpheres() {
  const points = useMemo(() => {
    return WAYPOINTS_2D.map(([x, z]) => {
      const y = getHeight(x, z) + 0.3;
      return new THREE.Vector3(x, y, z);
    });
  }, []);

  const curve = useMemo(() => new THREE.CatmullRomCurve3(points), [points]);
  const progressRef = useRef(0);
  const trailRefs = useRef<(THREE.Mesh | null)[]>(Array(TRAIL_COUNT).fill(null));
  const trailProgress = useRef<number[]>(Array(TRAIL_COUNT).fill(0));

  useFrame((_, delta) => {
    progressRef.current += delta * 0.08;
    if (progressRef.current > 1) progressRef.current -= 1;

    for (let i = 0; i < TRAIL_COUNT; i++) {
      const tp = progressRef.current - (i + 1) * 0.035;
      const clamped = tp < 0 ? tp + 1 : tp;
      const pos = curve.getPoint(clamped);
      const mesh = trailRefs.current[i];
      if (mesh) {
        mesh.position.copy(pos);
        mesh.scale.setScalar(0.08 * (1 - i / TRAIL_COUNT));
      }
    }
  });

  return (
    <>
      {Array.from({ length: TRAIL_COUNT }, (_, i) => (
        <mesh key={i} ref={(el) => { trailRefs.current[i] = el; }}>
          <sphereGeometry args={[1, 12, 12]} />
          <meshBasicMaterial
            color="#00ffcc"
            transparent
            opacity={0.35 * (1 - i / TRAIL_COUNT)}
            blending={THREE.AdditiveBlending}
            depthWrite={false}
          />
        </mesh>
      ))}
    </>
  );
}

function AttentionBeams() {
  const beams = useMemo(() => {
    const result: { start: THREE.Vector3; end: THREE.Vector3; phase: number }[] = [];
    for (let i = 0; i < WAYPOINTS_2D.length - 1; i++) {
      const [x1, z1] = WAYPOINTS_2D[i];
      const [x2, z2] = WAYPOINTS_2D[i + 1];
      const y1 = getHeight(x1, z1) + 0.3;
      const y2 = getHeight(x2, z2) + 0.3;
      result.push({
        start: new THREE.Vector3(x1, y1, z1),
        end: new THREE.Vector3(x2, y2, z2),
        phase: i * 0.5,
      });
    }
    return result;
  }, []);

  const beamRefs = useRef<(THREE.Mesh | null)[]>([]);

  useFrame((_, delta) => {
    const t = performance.now() / 1000;
    for (let i = 0; i < beams.length; i++) {
      const mesh = beamRefs.current[i];
      if (mesh) {
        const opacity = 0.15 + 0.2 * (Math.sin(t * 2 + beams[i].phase) * 0.5 + 0.5);
        (mesh.material as THREE.MeshBasicMaterial).opacity = opacity;
      }
    }
  });

  return (
    <>
      {beams.map((beam, i) => {
        const mid = new THREE.Vector3().addVectors(beam.start, beam.end).multiplyScalar(0.5);
        const dir = new THREE.Vector3().subVectors(beam.end, beam.start);
        const length = dir.length();
        dir.normalize();
        return (
          <mesh
            key={i}
            ref={(el) => { beamRefs.current[i] = el; }}
            position={mid}
            quaternion={new THREE.Quaternion().setFromUnitVectors(
              new THREE.Vector3(0, 1, 0), dir
            )}
          >
            <cylinderGeometry args={[0.01, 0.04, length, 4]} />
            <meshBasicMaterial
              color="#ff8844"
              transparent
              opacity={0.25}
              blending={THREE.AdditiveBlending}
              depthWrite={false}
            />
          </mesh>
        );
      })}
    </>
  );
}

interface ThinkingTrajectoryVisualizerProps {
  cycle?: number;
}

function Scene() {
  return (
    <>
      <color attach="background" args={['#0a0a14']} />
      <fogExp2 attach="fog" args={['#0a0a14', 0.025]} />

      <Terrain />
      <TrajectoryPath />
      <AttentionBeams />
      <TrailSpheres />
      <ambientLight intensity={0.3} />
      <directionalLight position={[5, 10, 5]} intensity={0.5} />
      <directionalLight position={[-5, 5, -5]} intensity={0.2} color="#4466ff" />

      <OrbitControls
        enableDamping
        dampingFactor={0.08}
        minDistance={5}
        maxDistance={25}
        target={[0, 0, 0]}
      />

      <EffectComposer>
        <Bloom luminanceThreshold={0.2} luminanceSmoothing={0.9} intensity={0.6} />
      </EffectComposer>
    </>
  );
}

export default function ThinkingTrajectoryVisualizer({ cycle }: ThinkingTrajectoryVisualizerProps) {
  return (
    <div style={{
      width: '100%', height: 420, borderRadius: 8, overflow: 'hidden',
      background: '#0a0a14', position: 'relative',
    }}>
      <Canvas
        camera={{ position: [10, 8, 10], fov: 50, near: 0.1, far: 40 }}
        gl={{ antialias: true, alpha: true }}
        dpr={[1, 2]}
      >
        <Scene />
      </Canvas>
      {cycle !== undefined && (
        <div style={{
          position: 'absolute', bottom: 8, right: 12,
          fontSize: 10, opacity: 0.4, fontFamily: 'monospace', color: '#888',
        }}>
          cycle #{cycle}
        </div>
      )}
    </div>
  );
}
