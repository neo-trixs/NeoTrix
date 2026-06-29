export async function readTextFile(path: string): Promise<string> {
  console.warn("[web-mock] readTextFile not available in browser mode", path);
  return "";
}

export async function writeTextFile(path: string, _contents: string): Promise<void> {
  console.warn("[web-mock] writeTextFile not available in browser mode", path);
}
