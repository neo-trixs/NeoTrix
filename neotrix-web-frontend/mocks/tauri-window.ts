interface WindowMock {
  getPosition(): Promise<{ x: number; y: number }>;
  getSize(): Promise<{ width: number; height: number }>;
  setPosition(_pos: { x: number; y: number }): Promise<void>;
  setSize(_size: { width: number; height: number }): Promise<void>;
  onResized(_fn: () => void): Promise<() => void>;
  onMoved(_fn: () => void): Promise<() => void>;
}

export function getCurrentWindow(): WindowMock {
  return {
    async getPosition() {
      return { x: 0, y: 0 };
    },
    async getSize() {
      return { width: 1280, height: 800 };
    },
    async setPosition(_pos: { x: number; y: number }) {},
    async setSize(_size: { width: number; height: number }) {},
    async onResized(_fn: () => void) {
      return () => {};
    },
    async onMoved(_fn: () => void) {
      return () => {};
    },
  };
}

export class PhysicalPosition {
  constructor(public x: number, public y: number) {}
}

export class PhysicalSize {
  constructor(public width: number, public height: number) {}
}
