export interface E8Root {
  vector: number[];
  index: number;
  isInteger: boolean;
  norm: number;
}

export function generateIntegerRoots(): number[][] {
  const roots: number[][] = [];
  for (let i = 0; i < 8; i++) {
    for (let j = i + 1; j < 8; j++) {
      for (const si of [-1, 1]) {
        for (const sj of [-1, 1]) {
          const v = new Array(8).fill(0);
          v[i] = si;
          v[j] = sj;
          roots.push(v);
        }
      }
    }
  }
  return roots;
}

export function generateHalfIntegerRoots(): number[][] {
  const roots: number[][] = [];
  for (let bits = 0; bits < 256; bits++) {
    let count = 0;
    for (let b = 0; b < 8; b++) {
      if (bits & (1 << b)) count++;
    }
    if (count % 2 === 0) {
      const v = new Array(8);
      for (let b = 0; b < 8; b++) {
        v[b] = (bits & (1 << b)) ? -0.5 : 0.5;
      }
      roots.push(v);
    }
  }
  return roots;
}

export function generateE8Roots(): E8Root[] {
  const integerVectors = generateIntegerRoots();
  const halfVectors = generateHalfIntegerRoots();
  const allVectors = [...integerVectors, ...halfVectors];
  return allVectors.map((v, i) => ({
    vector: v,
    index: i,
    isInteger: i < integerVectors.length,
    norm: Math.sqrt(v.reduce((s, x) => s + x * x, 0)),
  }));
}

export function projectTo3D(
  vector: number[],
  basis: number[][] = [
    [1, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 0, 0],
  ]
): [number, number, number] {
  return [
    vector.reduce((s, vi, i) => s + vi * basis[0][i], 0),
    vector.reduce((s, vi, i) => s + vi * basis[1][i], 0),
    vector.reduce((s, vi, i) => s + vi * basis[2][i], 0),
  ];
}

const φ = (1 + Math.sqrt(5)) / 2;
const φ2 = φ * φ;
const φ3 = φ2 * φ;

const PLANES: [number, number, number][] = [
  [0, 1, 1.0],
  [2, 3, φ],
  [4, 5, φ2],
  [6, 7, φ3],
];

export function rotateE8(vectors: number[][], time: number): number[][] {
  const result: number[][] = [];
  for (let k = 0; k < vectors.length; k++) {
    const v = vectors[k];
    const r = [...v];
    for (const [i, j, speed] of PLANES) {
      const θ = time * speed;
      const c = Math.cos(θ);
      const s = Math.sin(θ);
      const vi = r[i];
      const vj = r[j];
      r[i] = vi * c - vj * s;
      r[j] = vi * s + vj * c;
    }
    result.push(r);
  }
  return result;
}

export function findNeighborEdges(vectors: number[][]): [number, number][] {
  const edges: [number, number][] = [];
  const n = vectors.length;
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      let dot = 0;
      const vi = vectors[i];
      const vj = vectors[j];
      for (let k = 0; k < 8; k++) {
        dot += vi[k] * vj[k];
      }
      if (Math.abs(dot - 1) < 1e-6) {
        edges.push([i, j]);
      }
    }
  }
  return edges;
}
