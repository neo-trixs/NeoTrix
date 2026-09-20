declare module "@tauri-apps/api/core" {
  export function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  export class Resource {
    get rid(): number;
    constructor(rid: number);
    close(): Promise<void>;
  }
}
