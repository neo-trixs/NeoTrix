import * as THREE from 'three';

export interface HolographicUniforms {
  uTime: { value: number };
  uFresnelPower: { value: number };
  uGlitchIntensity: { value: number };
  uScanIntensity: { value: number };
  uColor: { value: THREE.Color };
  uEdgeColor: { value: THREE.Color };
}

const VERTEX_SHADER = `
varying vec3 vNormal;
varying vec3 vPosition;
varying vec2 vUv;

void main() {
  vNormal = normalize(normalMatrix * normal);
  vPosition = (modelViewMatrix * vec4(position, 1.0)).xyz;
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const FRAGMENT_SHADER = `
uniform float uTime;
uniform float uFresnelPower;
uniform float uGlitchIntensity;
uniform float uScanIntensity;
uniform vec3 uColor;
uniform vec3 uEdgeColor;

varying vec3 vNormal;
varying vec3 vPosition;
varying vec2 vUv;

void main() {
  vec3 viewDir = normalize(-vPosition);
  float fresnel = pow(1.0 - abs(dot(normalize(vNormal), viewDir)), uFresnelPower);

  float scanline = sin(vPosition.y * 40.0 + uTime * 2.0) * 0.5 + 0.5;
  scanline = pow(scanline, 4.0) * uScanIntensity;

  float glitch = 0.0;
  glitch += step(0.995, sin(vPosition.x * 50.0 + uTime * 3.0)) * uGlitchIntensity;
  glitch += step(0.995, sin(vPosition.z * 50.0 + uTime * 4.0)) * uGlitchIntensity;
  glitch = min(glitch, 0.8);

  vec3 color = mix(uColor, uEdgeColor, fresnel * 0.7);
  color += scanline * 0.25;
  color += vec3(0.0, 0.2, 0.5) * glitch;

  float alpha = 0.35 + 0.65 * fresnel + scanline * 0.15;
  gl_FragColor = vec4(color, alpha);
}
`;

export function createHolographicMaterial(
  params?: Partial<HolographicUniforms>
): THREE.ShaderMaterial {
  const defaults: HolographicUniforms = {
    uTime: { value: 0 },
    uFresnelPower: { value: 2.0 },
    uGlitchIntensity: { value: 0.05 },
    uScanIntensity: { value: 0.3 },
    uColor: { value: new THREE.Color('#00ffcc') },
    uEdgeColor: { value: new THREE.Color('#8844ff') },
  };

  const uniforms: Record<string, { value: any }> = {};
  for (const [key, val] of Object.entries(defaults)) {
    uniforms[key] = params?.[key as keyof HolographicUniforms] ?? val;
  }

  return new THREE.ShaderMaterial({
    uniforms,
    vertexShader: VERTEX_SHADER,
    fragmentShader: FRAGMENT_SHADER,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
}
