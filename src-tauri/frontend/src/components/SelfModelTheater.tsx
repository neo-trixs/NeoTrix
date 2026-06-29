import { useRef, useMemo, useEffect } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, MeshReflectorMaterial } from '@react-three/drei';
import * as THREE from 'three';
import { createHolographicMaterial } from '../engine/holographic-shader';

const PARTICLE_COUNT = 4096;
const SPHERE_RADIUS_MIN = 6;
const SPHERE_RADIUS_MAX = 10;

interface SelfModelTheaterProps {
  cycle?: number;
}

function ParticleField() {
  const meshRef = useRef<THREE.InstancedMesh>(null);
  const timeRef = useRef(0);

  const dummy = useMemo(() => new THREE.Object3D(), []);
  const material = useMemo(() => createHolographicMaterial({
    uFresnelPower: { value: 2.5 },
    uScanIntensity: { value: 0.2 },
    uGlitchIntensity: { value: 0.03 },
  }), []);

  const positions = useMemo(() => {
    const p: { pos: THREE.Vector3; scale: number; phase: number }[] = [];
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      const θ = Math.acos(2 * Math.random() - 1);
      const φ = Math.random() * Math.PI * 2;
      const r = SPHERE_RADIUS_MIN + Math.random() * (SPHERE_RADIUS_MAX - SPHERE_RADIUS_MIN);
      p.push({
        pos: new THREE.Vector3(
          r * Math.sin(θ) * Math.cos(φ),
          r * Math.sin(θ) * Math.sin(φ),
          r * Math.cos(θ)
        ),
        scale: 0.04 + Math.random() * 0.08,
        phase: Math.random() * Math.PI * 2,
      });
    }
    return p;
  }, []);

  useFrame((_, delta) => {
    timeRef.current += delta;
    const t = timeRef.current;

    material.uniforms.uTime.value = t;

    if (!meshRef.current) return;
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      const { pos, scale, phase } = positions[i];
      const pulse = 1 + Math.sin(t * 0.5 + phase) * 0.15;
      dummy.position.copy(pos);
      const angle = t * 0.03;
      dummy.position.applyAxisAngle(new THREE.Vector3(0, 1, 0), angle);
      dummy.position.applyAxisAngle(new THREE.Vector3(1, 0, 0), angle * 0.4);
      dummy.scale.setScalar(scale * pulse);
      dummy.rotation.set(
        Math.sin(t * 0.2 + phase) * 0.5,
        Math.cos(t * 0.3 + phase) * 0.5,
        0
      );
      dummy.updateMatrix();
      meshRef.current.setMatrixAt(i, dummy.matrix);
    }
    meshRef.current.instanceMatrix.needsUpdate = true;
  });

  const geometry = useMemo(() => new THREE.IcosahedronGeometry(1, 1), []);

  return (
    <instancedMesh
      ref={meshRef}
      args={[geometry, material, PARTICLE_COUNT]}
    />
  );
}

function Scene() {
  return (
    <>
      <color attach="background" args={['#0a0a14']} />
      <fogExp2 attach="fog" args={['#0a0a14', 0.018]} />

      <ParticleField />

      <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -5, 0]}>
        <planeGeometry args={[18, 12]} />
        <MeshReflectorMaterial
          blur={[400, 100]}
          resolution={512}
          mixBlur={6}
          mixStrength={1.2}
          color="#0a0a14"
          metalness={0.95}
          roughness={0.05}
        />
      </mesh>

      <ambientLight intensity={0.2} />
      <pointLight position={[0, 8, 0]} intensity={0.5} color="#00ffcc" />

      <OrbitControls
        enableDamping
        dampingFactor={0.08}
        minDistance={5}
        maxDistance={25}
        target={[0, 0, 0]}
      />
    </>
  );
}

export default function SelfModelTheater({ cycle }: SelfModelTheaterProps) {
  return (
    <div style={{
      width: '100%', height: 420, borderRadius: 8, overflow: 'hidden',
      background: '#0a0a14', position: 'relative',
    }}>
      <Canvas
        camera={{ position: [12, 8, 12], fov: 50, near: 0.1, far: 60 }}
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
