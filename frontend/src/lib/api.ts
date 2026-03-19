import type {
  FileRow,
  SymbolRow,
  GraphResponse,
  CommunityGraphResponse,
  Community,
  ProcessFlow,
  NarrativeResponse,
  Chapter,
  SourceResponse,
} from '$lib/types';

const BASE = '/api/v1';

// ── In-memory cache with TTL ──
const _cache = new Map<string, { data: unknown; expires: number }>();
const DEFAULT_CACHE_TTL = 5 * 60 * 1000; // 5 minutes

async function get<T>(url: string, fallback: T): Promise<T> {
  try {
    const res = await fetch(url);
    if (!res.ok) {
      console.error(`API ${url} returned ${res.status}`);
      return fallback;
    }
    const ct = res.headers.get('content-type') || '';
    if (!ct.includes('application/json')) {
      // SPA fallback returned HTML — endpoint doesn't exist
      return fallback;
    }
    return await res.json();
  } catch (e) {
    console.error(`API ${url} failed:`, e);
    return fallback;
  }
}

/** Cached GET — returns from in-memory cache if fresh, otherwise fetches. */
async function cachedGet<T>(url: string, fallback: T, ttl = DEFAULT_CACHE_TTL): Promise<T> {
  const entry = _cache.get(url);
  if (entry && entry.expires > Date.now()) {
    return entry.data as T;
  }
  const data = await get<T>(url, fallback);
  _cache.set(url, { data, expires: Date.now() + ttl });
  return data;
}

export async function fetchHealth(): Promise<{ status: string }> {
  return get(`${BASE}/health`, { status: 'disconnected' });
}

export async function fetchFiles(language?: string): Promise<FileRow[]> {
  const params = language ? `?language=${encodeURIComponent(language)}` : '';
  return cachedGet(`${BASE}/files${params}`, []);
}

export async function fetchFile(id: number): Promise<FileRow | null> {
  return get(`${BASE}/files/${id}`, null);
}

export async function fetchFileSymbols(fileId: number): Promise<SymbolRow[]> {
  return get(`${BASE}/files/${fileId}/symbols`, []);
}

export async function fetchSymbols(kind?: string): Promise<SymbolRow[]> {
  const params = kind ? `?kind=${encodeURIComponent(kind)}` : '';
  return cachedGet(`${BASE}/symbols${params}`, []);
}

export async function fetchSymbol(id: number): Promise<SymbolRow | null> {
  return get(`${BASE}/symbols/${id}`, null);
}

export async function searchSymbols(query: string): Promise<SymbolRow[]> {
  return get(`${BASE}/symbols/search?q=${encodeURIComponent(query)}`, []);
}

export async function fetchGraph(): Promise<GraphResponse> {
  return get(`${BASE}/graph`, { nodes: [], edges: [] });
}

export async function fetchCommunityGraph(): Promise<CommunityGraphResponse> {
  return cachedGet(`${BASE}/graph/communities`, { nodes: [], edges: [] });
}

export async function fetchCommunities(): Promise<Community[]> {
  return cachedGet(`${BASE}/communities`, []);
}

export async function fetchProcesses(): Promise<ProcessFlow[]> {
  return get(`${BASE}/processes`, []);
}

export async function fetchNarrative(kind: string): Promise<NarrativeResponse | null> {
  return cachedGet(`${BASE}/narratives/${kind}`, null);
}

export async function fetchNarrativeByTarget(kind: string, targetId: number): Promise<NarrativeResponse | null> {
  return get(`${BASE}/narratives/${kind}/${targetId}`, null);
}

export async function fetchChapters(): Promise<Chapter[]> {
  return cachedGet(`${BASE}/chapters`, []);
}

export async function fetchChapter(id: number): Promise<Chapter | null> {
  return get(`${BASE}/chapters/${id}`, null);
}

export async function fetchFileSource(fileId: number, start?: number, end?: number): Promise<SourceResponse | null> {
  const params = new URLSearchParams();
  if (start !== undefined) params.set('start', String(start));
  if (end !== undefined) params.set('end', String(end));
  const qs = params.toString() ? `?${params.toString()}` : '';
  return get(`${BASE}/files/${fileId}/source${qs}`, null);
}

export async function fetchLlmStatus(): Promise<{ available: boolean; provider: string }> {
  return get(`${BASE}/llm/status`, { available: false, provider: '' });
}

export async function fetchProgress(): Promise<import('$lib/types').Progress[]> {
  return get(`${BASE}/progress`, []);
}

export async function fetchQuiz(chapterId: number): Promise<import('$lib/types').QuizQuestion[]> {
  return get(`${BASE}/chapters/${chapterId}/quiz`, []);
}

export async function submitQuizAnswer(questionId: number, answer: string): Promise<import('$lib/types').QuizAnswerResult> {
  try {
    const res = await fetch(`${BASE}/quiz/${questionId}/answer`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ answer }),
    });
    if (!res.ok) return { correct: false, xp_earned: 0 };
    return await res.json();
  } catch {
    return { correct: false, xp_earned: 0 };
  }
}

export async function markSectionComplete(chapterId: number, sectionId: number): Promise<boolean> {
  try {
    const res = await fetch(`${BASE}/chapters/${chapterId}/sections/${sectionId}/complete`, {
      method: 'POST',
    });
    return res.ok;
  } catch {
    return false;
  }
}

export async function skipChapter(id: number): Promise<void> {
  await fetch(`${BASE}/chapters/${id}/skip`, { method: 'POST' });
}

export async function resetProgress(): Promise<void> {
  await fetch(`${BASE}/progress`, { method: 'DELETE' });
}

export async function fetchLearnerStats(): Promise<import('$lib/types').LearnerStats> {
  return get(`${BASE}/learner/stats`, {
    total_xp: 0,
    streak_days: 0,
    last_active: '',
    chapters_completed: 0,
    badges: [],
  });
}

// ── Annotations ──
export async function fetchAnnotations(flagged?: boolean): Promise<import('$lib/types').Annotation[]> {
  const params = flagged ? '?flagged=true' : '';
  return get(`${BASE}/annotations${params}`, []);
}

export async function fetchAnnotationsByTarget(targetType: string, targetId: number): Promise<import('$lib/types').Annotation[]> {
  return get(`${BASE}/annotations/${targetType}/${targetId}`, []);
}

export async function createAnnotation(targetType: string, targetId: number, content: string): Promise<import('$lib/types').Annotation | null> {
  try {
    const res = await fetch(`${BASE}/annotations`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ target_type: targetType, target_id: targetId, content }),
    });
    if (!res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

export async function updateAnnotation(id: number, content: string): Promise<boolean> {
  try {
    const res = await fetch(`${BASE}/annotations/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content }),
    });
    return res.ok;
  } catch {
    return false;
  }
}

export async function toggleAnnotationFlag(id: number): Promise<{ flagged: boolean } | null> {
  try {
    const res = await fetch(`${BASE}/annotations/${id}/flag`, { method: 'POST' });
    if (!res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

export async function deleteAnnotation(id: number): Promise<boolean> {
  try {
    const res = await fetch(`${BASE}/annotations/${id}`, { method: 'DELETE' });
    return res.ok;
  } catch {
    return false;
  }
}

export async function askQuestion(
  question: string,
  contextSymbolIds: number[],
  onDelta: (text: string) => void,
  onDone: () => void,
  onError: (msg: string) => void,
): Promise<void> {
  try {
    const res = await fetch(`${BASE}/ask`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ question, context_symbol_ids: contextSymbolIds }),
    });

    if (!res.ok) {
      const body = await res.json().catch(() => ({ content: 'Request failed' }));
      onError(body.content || `HTTP ${res.status}`);
      return;
    }

    const ct = res.headers.get('content-type') || '';
    if (ct.includes('text/event-stream')) {
      // SSE streaming
      const reader = res.body!.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            try {
              const data = JSON.parse(line.slice(6));
              if (data.type === 'delta') onDelta(data.content);
              else if (data.type === 'done') onDone();
              else if (data.type === 'error') onError(data.content);
            } catch { /* ignore parse errors */ }
          }
        }
      }
      onDone();
    } else {
      // Non-streaming JSON response (error case)
      const body = await res.json();
      if (body.type === 'error') onError(body.content);
      else onDone();
    }
  } catch (e) {
    onError(`Network error: ${e}`);
  }
}
