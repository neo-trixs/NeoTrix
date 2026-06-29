type TodoStatus = 'pending' | 'in_progress' | 'completed' | 'cancelled';
type TodoPriority = 'low' | 'medium' | 'high';

interface Todo {
  id: string;
  title: string;
  description: string;
  status: TodoStatus;
  priority: TodoPriority;
  createdAt: number;
  updatedAt: number;
  tags: string[];
}

export class TodoSystem {
  private todos: Map<string, Todo> = new Map();

  add(title: string, priority: TodoPriority = 'medium', tags: string[] = []): string {
    const id = `todo_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const todo: Todo = {
      id,
      title,
      description: '',
      status: 'pending',
      priority,
      createdAt: Date.now(),
      updatedAt: Date.now(),
      tags,
    };
    this.todos.set(id, todo);
    return id;
  }

  update(id: string, partial: Partial<Pick<Todo, 'title' | 'description' | 'status' | 'priority' | 'tags'>>): void {
    const todo = this.todos.get(id);
    if (!todo) return;
    Object.assign(todo, partial, { updatedAt: Date.now() });
  }

  setStatus(id: string, status: TodoStatus): void {
    this.update(id, { status });
  }

  delete(id: string): boolean {
    return this.todos.delete(id);
  }

  getById(id: string): Todo | undefined {
    return this.todos.get(id);
  }

  list(filter?: { status?: TodoStatus; priority?: TodoPriority }): Todo[] {
    let result = Array.from(this.todos.values());
    if (filter?.status) {
      result = result.filter(t => t.status === filter.status);
    }
    if (filter?.priority) {
      result = result.filter(t => t.priority === filter.priority);
    }
    result.sort((a, b) => b.createdAt - a.createdAt);
    return result;
  }

  getStats(): { total: number; pending: number; inProgress: number; completed: number } {
    const all = Array.from(this.todos.values());
    return {
      total: all.length,
      pending: all.filter(t => t.status === 'pending').length,
      inProgress: all.filter(t => t.status === 'in_progress').length,
      completed: all.filter(t => t.status === 'completed').length,
    };
  }

  size(): number {
    return this.todos.size;
  }

  summary(): string {
    const s = this.getStats();
    return `TodoSystem: ${s.total} total, ${s.pending} pending, ${s.inProgress} in-progress, ${s.completed} completed`;
  }
}
