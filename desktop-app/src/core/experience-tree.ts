interface ExperienceNode {
  id: string;
  label: string;
  description: string;
  confidence: number;
  createdAt: number;
  lastAccessedAt: number;
  accessCount: number;
  children: string[];
}

export class ExperienceTree {
  private nodes: Map<string, ExperienceNode> = new Map();
  private rootIds: string[] = [];

  addNode(
    id: string,
    label: string,
    description: string,
    confidence: number,
    parentId?: string,
  ): void {
    const node: ExperienceNode = {
      id,
      label,
      description,
      confidence: Math.max(0, Math.min(1, confidence)),
      createdAt: Date.now(),
      lastAccessedAt: Date.now(),
      accessCount: 0,
      children: [],
    };
    this.nodes.set(id, node);

    if (parentId && this.nodes.has(parentId)) {
      this.nodes.get(parentId)!.children.push(id);
    } else {
      this.rootIds.push(id);
    }
  }

  getNode(id: string): ExperienceNode | undefined {
    const node = this.nodes.get(id);
    if (node) {
      node.accessCount++;
      node.lastAccessedAt = Date.now();
    }
    return node;
  }

  decay(rate = 0.05): void {
    for (const node of this.nodes.values()) {
      node.confidence = Math.max(0, node.confidence - rate);
    }
  }

  reinforce(id: string, amount = 0.1): void {
    const node = this.nodes.get(id);
    if (node) {
      node.confidence = Math.min(1, node.confidence + amount);
    }
  }

  prune(threshold = 0.1): string[] {
    const removed: string[] = [];
    for (const [id, node] of this.nodes) {
      if (node.confidence < threshold && node.children.length === 0) {
        this.nodes.delete(id);
        removed.push(id);
      }
    }
    this.rootIds = this.rootIds.filter(id => this.nodes.has(id));
    return removed;
  }

  getRoots(): ExperienceNode[] {
    return this.rootIds.map(id => this.nodes.get(id)!).filter(Boolean);
  }

  size(): number {
    return this.nodes.size;
  }

  summary(): string {
    return `ExperienceTree: ${this.size()} nodes, ${this.getRoots().length} roots`;
  }
}
