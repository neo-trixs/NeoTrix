interface OpenOptions {
  multiple?: boolean;
  directories?: boolean;
  filters?: { name: string; extensions: string[] }[];
  defaultPath?: string;
}

export async function open(options?: OpenOptions): Promise<string | string[] | null> {
  console.warn("[web-mock] open dialog not available in browser mode", options);
  return null;
}

export async function save(options?: { defaultPath?: string }): Promise<string | null> {
  console.warn("[web-mock] save dialog not available in browser mode", options);
  return null;
}
