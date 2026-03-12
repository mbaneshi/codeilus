const BASE = '/api/v1';

export async function fetchHealth(): Promise<{ status: string }> {
  const res = await fetch(`${BASE}/health`);
  return res.json();
}

export async function fetchFiles(): Promise<unknown[]> {
  const res = await fetch(`${BASE}/files`);
  return res.json();
}

export async function fetchSymbols(fileId: number): Promise<unknown[]> {
  const res = await fetch(`${BASE}/files/${fileId}/symbols`);
  return res.json();
}
