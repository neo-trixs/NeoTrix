// ===== CodeGraphIntegrator — Structural Indexing + Multi-Phase Absorption =====
// Inspired by: RPG-Encoder (ICML 2026), Codebase-Memory, AOCI, LARGER, OpenLore
// D3 fix: cross-session persistence via localStorage
// D5 fix: confidence-scored retrieval with threshold

export type CodeEntityType = 'file' | 'module' | 'function' | 'class' | 'type' | 'import' | 'call' | 'route' | 'config';

export interface CodeEntity {
  id: string;
  type: CodeEntityType;
  name: string;
  path: string;
  lineStart: number;
  lineEnd: number;
  signature?: string;
  semanticFeatures?: string;
  confidence: number;
  tags: string[];
}

export type EdgeConfidence = 'EXTRACTED' | 'INFERRED' | 'AMBIGUOUS';
export type EdgeType = 'imports' | 'calls' | 'extends' | 'implements' | 'contains' | 'composes' | 'tests' | 'references';

export interface CodeEdge {
  sourceId: string;
  targetId: string;
  type: EdgeType;
  confidence: EdgeConfidence;
}

export type AbsorbSourceType = 'paper' | 'repo' | 'architecture' | 'pattern' | 'defect' | 'evolution' | 'conversation';

export interface AbsorbedKnowledge {
  id: string;
  title: string;
  description: string;
  sourceType: AbsorbSourceType;
  sourceIds: string[];
  sourceExcerpt: string;
  sourceCredibility: number;
  insights: string[];
  downstreamDefects: string[];
  createdAt: number;
  confidence: number;
  phaseCompleted: string[]; // tracks what absorption phases finished
}

export interface AbsorptionResult {
  knowledgeId: string;
  summary: string;
  entitiesFound: number;
  edgesFound: number;
  insightsExtracted: string[];
  defectsIdentified: string[];
  quality: number; // 0-1
}

export interface RepoSnapshot {
  name: string;
  entityCount: number;
  edgeCount: number;
  knowledgeCount: number;
  lastUpdated: number;
  coverage: { files: number; modules: number; functions: number };
  defects: { total: number; critical: number; fixed: number };
}

const ENTITY_COLORS: Record<CodeEntityType, string> = {
  file: '#6c8cff', module: '#8a6cff', function: '#ff6cb4',
  class: '#ffb86c', type: '#4caf7d', import: '#6ccfff',
  call: '#ff6c6c', route: '#c86cff', config: '#6cffd4',
};

const ABSORPTION_PHASES = ['scout', 'extract', 'deep-read', 'synthesize'] as const;
export type AbsorptionPhase = (typeof ABSORPTION_PHASES)[number];

export class CodeGraphIntegrator {
  private entities = new Map<string, CodeEntity>();
  private edges: CodeEdge[] = [];
  private knowledge = new Map<string, AbsorbedKnowledge>();
  private phaseLog: Array<{ phase: AbsorptionPhase; ts: number; detail: string }> = [];

  addEntity(entity: CodeEntity): void {
    this.entities.set(entity.id, entity);
  }

  addEdge(edge: CodeEdge): void {
    this.edges.push(edge);
  }

  addKnowledge(k: AbsorbedKnowledge): void {
    this.knowledge.set(k.id, k);
  }

  getEntity(id: string): CodeEntity | undefined {
    return this.entities.get(id);
  }

  getKnowledge(id: string): AbsorbedKnowledge | undefined {
    return this.knowledge.get(id);
  }

  getEntitiesByType(type: CodeEntityType): CodeEntity[] {
    return Array.from(this.entities.values()).filter(e => e.type === type);
  }

  getEdges(sourceId?: string, type?: EdgeType): CodeEdge[] {
    let result = this.edges;
    if (sourceId) result = result.filter(e => e.sourceId === sourceId);
    if (type) result = result.filter(e => e.type === type);
    return result;
  }

  getKnowledgeBySource(type: AbsorbSourceType): AbsorbedKnowledge[] {
    return Array.from(this.knowledge.values()).filter(k => k.sourceType === type);
  }

  getEntityColor(type: CodeEntityType): string {
    return ENTITY_COLORS[type] || '#999';
  }

  // ===== Multi-Phase Absorption Pipeline =====
  // Phase 1: Scout — structural scan
  scout(sources: Array<{ path: string; type: CodeEntityType; name: string }>): AbsorptionResult {
    const id = `abs_${Date.now()}`;
    const entities: CodeEntity[] = [];
    const edges: CodeEdge[] = [];
    let lastParent = '';

    for (const src of sources) {
      const eid = `ent_${src.path.replace(/[/\\:.]/g, '_')}`;
      entities.push({
        id: eid,
        type: src.type,
        name: src.name,
        path: src.path,
        lineStart: 0,
        lineEnd: 0,
        confidence: 0.6,
        tags: ['scout'],
      });
      if (lastParent) {
        edges.push({ sourceId: lastParent, targetId: eid, type: 'contains', confidence: 'EXTRACTED' });
      }
      lastParent = eid;
    }

    for (const e of entities) this.addEntity(e);
    for (const e of edges) this.addEdge(e);

    this.phaseLog.push({ phase: 'scout', ts: Date.now(), detail: `Found ${entities.length} entities, ${edges.length} edges` });

    return {
      knowledgeId: id,
      summary: `Scout: ${entities.length} entities mapped`,
      entitiesFound: entities.length,
      edgesFound: edges.length,
      insightsExtracted: [],
      defectsIdentified: [],
      quality: 0.6,
    };
  }

  // Phase 2: Extract — semantic lifting
  extract(_entityIds: string[], semanticMap: Map<string, string>): AbsorptionResult {
    let count = 0;
    for (const [eid, features] of semanticMap) {
      const ent = this.entities.get(eid);
      if (ent) {
        ent.semanticFeatures = features;
        ent.confidence = Math.min(1, ent.confidence + 0.2);
        ent.tags.push('extracted');
        count++;
      }
    }
    this.phaseLog.push({ phase: 'extract', ts: Date.now(), detail: `Lifted semantics for ${count} entities` });
    return {
      knowledgeId: `ext_${Date.now()}`,
      summary: `Extract: semantic features for ${count} entities`,
      entitiesFound: count,
      edgesFound: 0,
      insightsExtracted: [],
      defectsIdentified: [],
      quality: 0.75,
    };
  }

  // Phase 3: Deep-read — full-text consumption
  deepRead(targetIds: string[], fullTexts: Map<string, string>): AbsorptionResult {
    const insights: string[] = [];
    for (const [eid, text] of fullTexts) {
      const ent = this.entities.get(eid);
      if (ent) {
        ent.confidence = Math.min(1, ent.confidence + 0.3);
        ent.tags.push('deep-read');
        insights.push(`${ent.name}: ${text.slice(0, 80)}...`);
      }
    }
    this.phaseLog.push({ phase: 'deep-read', ts: Date.now(), detail: `Deep-read ${targetIds.length} high-value targets` });
    return {
      knowledgeId: `deep_${Date.now()}`,
      summary: `Deep-read: ${targetIds.length} targets consumed`,
      entitiesFound: targetIds.length,
      edgesFound: 0,
      insightsExtracted: insights,
      defectsIdentified: [],
      quality: 0.85,
    };
  }

  // Phase 4: Synthesize — cross-reference & pattern extraction
  synthesize(knowledgeEntries: AbsorbedKnowledge[]): AbsorptionResult {
    const defects: string[] = [];
    for (const k of knowledgeEntries) {
      this.addKnowledge(k);
      if (k.downstreamDefects.length > 0) {
        defects.push(...k.downstreamDefects);
      }
    }
    this.phaseLog.push({ phase: 'synthesize', ts: Date.now(), detail: `Synthesized ${knowledgeEntries.length} knowledge entries, ${defects.length} defects` });
    return {
      knowledgeId: `syn_${Date.now()}`,
      summary: `Synthesize: ${knowledgeEntries.length} entries, ${defects.length} defects linked`,
      entitiesFound: 0,
      edgesFound: 0,
      insightsExtracted: knowledgeEntries.map(k => k.title),
      defectsIdentified: defects,
      quality: 0.9,
    };
  }

  // Full pipeline run (auto-persist after completion)
  runAbsorptionPipeline(
    sources: Array<{ path: string; type: CodeEntityType; name: string }>,
    semanticMap: Map<string, string>,
    fullTexts: Map<string, string>,
    knowledgeEntries: AbsorbedKnowledge[],
  ): AbsorptionResult[] {
    const results = [
      this.scout(sources),
      this.extract(Array.from(semanticMap.keys()), semanticMap),
      this.deepRead(Array.from(fullTexts.keys()), fullTexts),
      this.synthesize(knowledgeEntries),
    ];
    this.persist();
    return results;
  }

  getPhases(): Array<{ phase: AbsorptionPhase; ts: number; detail: string }> {
    return this.phaseLog;
  }

  // ===== Cross-session persistence (D3) =====
  private STORAGE_KEY = 'neotrix_codegraph';

  persist(): void {
    try {
      const data = {
        entities: Array.from(this.entities.entries()),
        edges: this.edges,
        knowledge: Array.from(this.knowledge.entries()),
        phaseLog: this.phaseLog,
      };
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(data));
    } catch (e) { if (e instanceof Error) console.warn('[CodeGraph]', e.message); }
  }

  restore(): boolean {
    try {
      const raw = localStorage.getItem(this.STORAGE_KEY);
      if (!raw) return false;
      const data = JSON.parse(raw);
      this.entities = new Map(data.entities);
      this.edges = data.edges;
      this.knowledge = new Map(data.knowledge);
      this.phaseLog = data.phaseLog;
      return true;
    } catch (e) { if (e instanceof Error) console.warn('[CodeGraph]', e.message); return false; }
  }

  clearPersisted(): void {
    try { localStorage.removeItem(this.STORAGE_KEY); } catch (e) { if (e instanceof Error) console.warn('[CodeGraph]', e.message); }
  }

  // ===== Confidence-scored retrieval (D5) =====
  queryByConfidence(type: CodeEntityType | null, minConfidence: number, tags?: string[]): CodeEntity[] {
    let results = Array.from(this.entities.values());
    if (type) results = results.filter(e => e.type === type);
    if (minConfidence > 0) results = results.filter(e => e.confidence >= minConfidence);
    if (tags && tags.length > 0) results = results.filter(e => tags.some(t => e.tags.includes(t)));
    return results.sort((a, b) => b.confidence - a.confidence);
  }

  getKnowledgeByConfidence(minConfidence: number, sourceType?: AbsorbSourceType): AbsorbedKnowledge[] {
    let results = Array.from(this.knowledge.values());
    if (sourceType) results = results.filter(k => k.sourceType === sourceType);
    return results.filter(k => k.confidence >= minConfidence).sort((a, b) => b.confidence - a.confidence);
  }

  getStats(): RepoSnapshot {
    const coverage = {
      files: this.getEntitiesByType('file').length,
      modules: this.getEntitiesByType('module').length,
      functions: this.getEntitiesByType('function').length + this.getEntitiesByType('class').length,
    };
    const allK = Array.from(this.knowledge.values());
    const criticalDefects = allK.filter(k => k.downstreamDefects.length > 2).length;
    return {
      name: 'neotrix-core',
      entityCount: this.entities.size,
      edgeCount: this.edges.length,
      knowledgeCount: this.knowledge.size,
      lastUpdated: Date.now(),
      coverage,
      defects: { total: allK.length, critical: criticalDefects, fixed: 0 },
    };
  }

  summary(): string {
    const s = this.getStats();
    const phases = this.phaseLog.map(p => `${p.phase}[${new Date(p.ts).toLocaleTimeString()}]`).join(' → ');
    return `CodeGraph: ${s.entityCount} entities, ${s.edgeCount} edges, ${s.knowledgeCount} knowledge · ${phases}`;
  }
}

// Singleton (auto-restore on init)
export const codeGraph = (() => {
  const g = new CodeGraphIntegrator();
  g.restore();
  return g;
})();
