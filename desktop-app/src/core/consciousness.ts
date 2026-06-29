export interface CycleMetrics {
  step: string;
  duration: number;
  accuracy: number;
  novelty: number;
  timestamp: number;
}

export interface IntegrationSignal {
  source: string;
  type: string;
  strength: number;
  timestamp: number;
}

export interface FeedbackLoop {
  id: string;
  label: string;
  source: string;
  target: string;
  active: boolean;
  strength: number;
}

const DEFAULT_LOOPS: FeedbackLoop[] = [
  { id: 'f1', label: 'TemporalPrediction → FreeEnergy', source: 'TemporalPrediction', target: 'FreeEnergyCuriosity', active: true, strength: 0.85 },
  { id: 'f2', label: 'FreeEnergy → BoredomSignal', source: 'FreeEnergyCuriosity', target: 'BoredomSignal', active: true, strength: 0.72 },
  { id: 'f3', label: 'IIT Phi → MetaSEAL', source: 'IitPhi8Engine', target: 'MetaSealEngine', active: true, strength: 0.68 },
];

const DEFAULT_SIGNALS: IntegrationSignal[] = [
  { source: 'HebbianMemory', type: 'Divergence', strength: 0.4, timestamp: Date.now() },
  { source: 'FreeEnergy', type: 'Curiosity', strength: 0.7, timestamp: Date.now() - 1000 },
  { source: 'IitPhi8', type: 'Phi', strength: 0.55, timestamp: Date.now() - 2000 },
];

export function getDefaultLoops(): FeedbackLoop[] {
  return DEFAULT_LOOPS;
}

export function getDefaultSignals(): IntegrationSignal[] {
  return DEFAULT_SIGNALS.map(s => ({ ...s, timestamp: Date.now() }));
}

export type RunState = 'idle' | 'gather' | 'reflect' | 'reason' | 'plan' | 'act' | 'record';
