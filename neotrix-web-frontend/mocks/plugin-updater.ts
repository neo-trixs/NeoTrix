interface UpdateMock {
  version: string;
  available: boolean;
  download(_fn?: (event: { event: string; data: { progress?: number; total?: number } }) => void): Promise<void>;
  install(): Promise<void>;
}

export async function check(): Promise<UpdateMock | null> {
  return null;
}
