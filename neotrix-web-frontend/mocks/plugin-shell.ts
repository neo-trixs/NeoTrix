interface ChildMock {
  pid: number;
  stdout: string;
  stderr: string;
}

export async function execute(
  _program: string,
  _args?: string[],
  _options?: { cwd?: string }
): Promise<ChildMock> {
  console.warn("[web-mock] shell execute not available in browser mode");
  return { pid: 0, stdout: "", stderr: "" };
}
