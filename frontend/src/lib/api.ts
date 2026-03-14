import type {
  FileRow,
  SymbolRow,
  GraphResponse,
  Community,
  ProcessFlow,
} from '$lib/types';

const BASE = '/api/v1';

export async function fetchHealth(): Promise<{ status: string }> {
  try {
    const res = await fetch(`${BASE}/health`);
    return await res.json();
  } catch (e) {
    console.error('fetchHealth failed:', e);
    return { status: 'disconnected' };
  }
}

export async function fetchFiles(language?: string): Promise<FileRow[]> {
  try {
    const params = language ? `?language=${encodeURIComponent(language)}` : '';
    const res = await fetch(`${BASE}/files${params}`);
    return await res.json();
  } catch (e) {
    console.error('fetchFiles failed:', e);
    return [];
  }
}

export async function fetchFile(id: number): Promise<FileRow | null> {
  try {
    const res = await fetch(`${BASE}/files/${id}`);
    return await res.json();
  } catch (e) {
    console.error('fetchFile failed:', e);
    return null;
  }
}

export async function fetchFileSymbols(fileId: number): Promise<SymbolRow[]> {
  try {
    const res = await fetch(`${BASE}/files/${fileId}/symbols`);
    return await res.json();
  } catch (e) {
    console.error('fetchFileSymbols failed:', e);
    return [];
  }
}

export async function fetchSymbols(kind?: string): Promise<SymbolRow[]> {
  try {
    const params = kind ? `?kind=${encodeURIComponent(kind)}` : '';
    const res = await fetch(`${BASE}/symbols${params}`);
    return await res.json();
  } catch (e) {
    console.error('fetchSymbols failed:', e);
    return [];
  }
}

export async function fetchSymbol(id: number): Promise<SymbolRow | null> {
  try {
    const res = await fetch(`${BASE}/symbols/${id}`);
    return await res.json();
  } catch (e) {
    console.error('fetchSymbol failed:', e);
    return null;
  }
}

export async function searchSymbols(query: string): Promise<SymbolRow[]> {
  try {
    const res = await fetch(`${BASE}/symbols/search?q=${encodeURIComponent(query)}`);
    return await res.json();
  } catch (e) {
    console.error('searchSymbols failed:', e);
    return [];
  }
}

export async function fetchGraph(): Promise<GraphResponse> {
  try {
    const res = await fetch(`${BASE}/graph`);
    return await res.json();
  } catch (e) {
    console.error('fetchGraph failed:', e);
    return { nodes: [], edges: [] };
  }
}

export async function fetchCommunities(): Promise<Community[]> {
  try {
    const res = await fetch(`${BASE}/communities`);
    return await res.json();
  } catch (e) {
    console.error('fetchCommunities failed:', e);
    return [];
  }
}

export async function fetchProcesses(): Promise<ProcessFlow[]> {
  try {
    const res = await fetch(`${BASE}/processes`);
    return await res.json();
  } catch (e) {
    console.error('fetchProcesses failed:', e);
    return [];
  }
}
