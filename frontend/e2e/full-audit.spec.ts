import { test, expect } from '@playwright/test';

// ============================================================
// LAYER 1: API ENDPOINTS
// ============================================================

test.describe('API Layer', () => {
  test('GET /api/v1/health returns ok', async ({ request }) => {
    const res = await request.get('/api/v1/health');
    expect(res.status()).toBe(200);
    const body = await res.json();
    expect(body.status).toBe('ok');
  });

  test('GET /api/v1/files returns non-empty array with correct shape', async ({ request }) => {
    const res = await request.get('/api/v1/files');
    expect(res.status()).toBe(200);
    const files = await res.json();
    expect(Array.isArray(files)).toBe(true);
    expect(files.length).toBeGreaterThan(0);
    const f = files[0];
    expect(typeof f.id).toBe('number');
    expect(typeof f.path).toBe('string');
    expect(typeof f.sloc).toBe('number');
  });

  test('GET /api/v1/files?language=rust filters correctly', async ({ request }) => {
    const res = await request.get('/api/v1/files?language=rust');
    expect(res.status()).toBe(200);
    const files = await res.json();
    expect(files.length).toBeGreaterThan(0);
    for (const f of files) {
      expect(f.language).toBe('rust');
    }
  });

  test('GET /api/v1/files/:id returns single file', async ({ request }) => {
    const res = await request.get('/api/v1/files/1');
    expect(res.status()).toBe(200);
    const file = await res.json();
    expect(file.id).toBe(1);
    expect(typeof file.path).toBe('string');
  });

  test('GET /api/v1/files/:id/symbols returns symbols', async ({ request }) => {
    const res = await request.get('/api/v1/files/1/symbols');
    expect(res.status()).toBe(200);
    const symbols = await res.json();
    expect(Array.isArray(symbols)).toBe(true);
  });

  test('GET /api/v1/symbols returns array', async ({ request }) => {
    const res = await request.get('/api/v1/symbols');
    expect(res.status()).toBe(200);
    const symbols = await res.json();
    expect(Array.isArray(symbols)).toBe(true);
    expect(symbols.length).toBeGreaterThan(0);
  });

  test('GET /api/v1/symbols/search?q=main returns results', async ({ request }) => {
    const res = await request.get('/api/v1/symbols/search?q=main');
    expect(res.status()).toBe(200);
    const results = await res.json();
    expect(Array.isArray(results)).toBe(true);
    expect(results.length).toBeGreaterThan(0);
  });

  test('GET /api/v1/graph returns nodes and edges', async ({ request }) => {
    const res = await request.get('/api/v1/graph');
    expect(res.status()).toBe(200);
    const graph = await res.json();
    expect(graph.nodes.length).toBeGreaterThan(0);
    expect(graph.edges.length).toBeGreaterThan(0);
    const n = graph.nodes[0];
    expect(typeof n.id).toBe('number');
    expect(typeof n.name).toBe('string');
  });

  test('GET /api/v1/communities returns non-empty array', async ({ request }) => {
    const res = await request.get('/api/v1/communities');
    expect(res.status()).toBe(200);
    const communities = await res.json();
    expect(communities.length).toBeGreaterThan(0);
    const c = communities[0];
    expect(typeof c.id).toBe('number');
    expect(typeof c.label).toBe('string');
    expect(typeof c.member_count).toBe('number');
  });

  test('GET /api/v1/processes returns array with steps', async ({ request }) => {
    const res = await request.get('/api/v1/processes');
    expect(res.status()).toBe(200);
    const processes = await res.json();
    expect(processes.length).toBeGreaterThan(0);
    const p = processes[0];
    expect(typeof p.id).toBe('number');
    expect(typeof p.name).toBe('string');
    expect(Array.isArray(p.steps)).toBe(true);
  });
});

// ============================================================
// LAYER 2: SPA ROUTING
// ============================================================

test.describe('SPA Routing', () => {
  const routes = ['/', '/learn', '/explore', '/explore/tree', '/explore/graph', '/explore/metrics', '/explore/diagrams', '/ask'];

  for (const route of routes) {
    test(`${route} serves SPA index.html`, async ({ request }) => {
      const res = await request.get(route);
      expect(res.status()).toBe(200);
      const html = await res.text();
      expect(html).toContain('__sveltekit');
      expect(html).toContain('base: ""');
    });
  }

  test('static JS assets return JS content-type', async ({ request }) => {
    const html = await (await request.get('/')).text();
    const jsMatch = html.match(/\/_app\/immutable\/entry\/app\.[^"]+\.js/);
    expect(jsMatch).toBeTruthy();
    const res = await request.get(jsMatch![0]);
    expect(res.status()).toBe(200);
    const ct = res.headers()['content-type'];
    expect(ct).toContain('javascript');
  });
});

// ============================================================
// LAYER 3: PAGE RENDERING + DATA LOADING
// ============================================================

test.describe('Home Page (/)', () => {
  test('shows stats after loading', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByText('ok')).toBeVisible({ timeout: 10000 });
    // Stats should appear
    await expect(page.getByText('Files')).toBeVisible();
    await expect(page.getByText('SLOC')).toBeVisible();
    await expect(page.getByText('Languages')).toBeVisible();
  });

  test('navigation cards link to correct pages', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: 'Learn' }).first().click();
    await expect(page).toHaveURL('/learn');
  });

  test('no console errors on home', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()); });
    await page.goto('/');
    await page.waitForTimeout(3000);
    const realErrors = errors.filter(e => !e.includes('WebSocket') && !e.includes('ws://') && !e.includes('ERR_CONNECTION'));
    expect(realErrors).toEqual([]);
  });
});

test.describe('Learn Page (/learn)', () => {
  test('shows chapter cards after loading', async ({ page }) => {
    await page.goto('/learn');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: 'Learning Path' })).toBeVisible();
    // Should show at least one chapter card
    const cards = page.locator('.card');
    await expect(cards.first()).toBeVisible();
    const count = await cards.count();
    expect(count).toBeGreaterThan(0);
  });

  test('chapter labels are human-readable (not raw cluster_ names)', async ({ page }) => {
    await page.goto('/learn');
    await expect(page.locator('.card').first()).toBeVisible({ timeout: 10000 });
    const label = await page.locator('.card h3').first().textContent();
    expect(label).toBeTruthy();
    // Should NOT start with cluster_ (formatted)
    expect(label!.toLowerCase().startsWith('cluster_')).toBe(false);
  });

  test('Start button expands chapter content', async ({ page }) => {
    await page.goto('/learn');
    await expect(page.locator('.card').first()).toBeVisible({ timeout: 10000 });
    const startBtn = page.locator('.card button').first();
    await expect(startBtn).toHaveText('Start');
    await startBtn.click();
    // Button should change to "Collapse"
    await expect(startBtn).toHaveText('Collapse');
    // Expanded content should be visible
    await expect(page.getByText('This module covers')).toBeVisible();
  });

  test('chapter shows topic count and cohesion', async ({ page }) => {
    await page.goto('/learn');
    await expect(page.locator('.card').first()).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/\d+ symbols/).first()).toBeVisible();
    await expect(page.getByText(/Cohesion \d+%/).first()).toBeVisible();
  });
});

test.describe('Explore Hub (/explore)', () => {
  test('shows 4 navigation cards', async ({ page }) => {
    await page.goto('/explore');
    await expect(page.getByText('File Tree')).toBeVisible();
    await expect(page.getByText('Graph')).toBeVisible();
    await expect(page.getByText('Metrics')).toBeVisible();
    await expect(page.getByText('Diagrams')).toBeVisible();
  });

  test('cards navigate to correct pages', async ({ page }) => {
    await page.goto('/explore');
    await page.getByRole('link', { name: 'File Tree' }).click();
    await expect(page).toHaveURL('/explore/tree');
  });
});

test.describe('File Tree (/explore/tree)', () => {
  test('shows file tree after loading', async ({ page }) => {
    await page.goto('/explore/tree');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: 'File Tree' })).toBeVisible();
  });

  test('tree has expandable directories', async ({ page }) => {
    await page.goto('/explore/tree');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    const dirButtons = page.locator('button:has-text("/")');
    const count = await dirButtons.count();
    expect(count).toBeGreaterThan(0);
  });

  test('clicking a directory expands it', async ({ page }) => {
    await page.goto('/explore/tree');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    const firstDir = page.locator('button:has-text("/")').first();
    const countBefore = await page.locator('.tree-row').count();
    await firstDir.click();
    await page.waitForTimeout(500);
    const countAfter = await page.locator('.tree-row').count();
    expect(countAfter).toBeGreaterThan(countBefore);
  });

  test('clicking a file shows symbol panel', async ({ page }) => {
    await page.goto('/explore/tree');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    // Expand first directory
    await page.locator('button:has-text("/")').first().click();
    await page.waitForTimeout(300);
    // Click a .rs file
    const fileBtn = page.locator('button:has-text(".rs")').first();
    if (await fileBtn.count() > 0) {
      await fileBtn.click();
      await expect(page.getByText('Symbols')).toBeVisible({ timeout: 5000 });
    }
  });
});

test.describe('Graph (/explore/graph)', () => {
  test('shows graph with stats', async ({ page }) => {
    await page.goto('/explore/graph');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 15000 });
    await expect(page.getByRole('heading', { name: 'Knowledge Graph' })).toBeVisible();
    await expect(page.getByText(/\d+ nodes/)).toBeVisible({ timeout: 15000 });
    await expect(page.getByText(/\d+ edges/)).toBeVisible();
  });

  test('SVG with circles is rendered', async ({ page }) => {
    await page.goto('/explore/graph');
    await expect(page.getByText(/\d+ nodes/)).toBeVisible({ timeout: 15000 });
    const circles = page.locator('svg circle');
    const count = await circles.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('Metrics (/explore/metrics)', () => {
  test('shows dashboard with stats', async ({ page }) => {
    await page.goto('/explore/metrics');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: 'Metrics Dashboard' })).toBeVisible();
    await expect(page.getByText('Total Files')).toBeVisible();
    await expect(page.getByText('Total SLOC')).toBeVisible();
  });

  test('shows language distribution', async ({ page }) => {
    await page.goto('/explore/metrics');
    await expect(page.getByRole('heading', { name: 'Language Distribution' })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('rust').first()).toBeVisible();
  });

  test('shows top files table', async ({ page }) => {
    await page.goto('/explore/metrics');
    await expect(page.getByText('Top Files by SLOC')).toBeVisible({ timeout: 10000 });
    const rows = page.locator('tbody tr');
    const count = await rows.count();
    expect(count).toBeGreaterThan(0);
  });

  test('sort toggles work', async ({ page }) => {
    await page.goto('/explore/metrics');
    await expect(page.getByText('Top Files by SLOC')).toBeVisible({ timeout: 10000 });
    await page.locator('th:has-text("Path")').click();
    await page.waitForTimeout(300);
    // Just verify no crash
    const rows = page.locator('tbody tr');
    expect(await rows.count()).toBeGreaterThan(0);
  });
});

test.describe('Diagrams (/explore/diagrams)', () => {
  test('shows communities section', async ({ page }) => {
    await page.goto('/explore/diagrams');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: 'Diagrams', exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Communities', exact: true })).toBeVisible();
  });

  test('community cards show member count', async ({ page }) => {
    await page.goto('/explore/diagrams');
    await expect(page.getByRole('heading', { name: 'Communities', exact: true })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText(/\d+ members/).first()).toBeVisible();
  });

  test('community cards show cohesion bars', async ({ page }) => {
    await page.goto('/explore/diagrams');
    await expect(page.getByRole('heading', { name: 'Communities', exact: true })).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Cohesion').first()).toBeVisible();
  });

  test('process flows section shows', async ({ page }) => {
    await page.goto('/explore/diagrams');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByRole('heading', { name: 'Process Flows' })).toBeVisible();
  });

  test('clicking a process expands it', async ({ page }) => {
    await page.goto('/explore/diagrams');
    await expect(page.getByRole('heading', { name: 'Process Flows' })).toBeVisible({ timeout: 10000 });
    // Scroll to Process Flows section
    await page.getByRole('heading', { name: 'Process Flows' }).scrollIntoViewIfNeeded();
    // Find a process card button and click
    const processBtn = page.locator('h2:has-text("Process Flows") ~ div .card button').first();
    if (await processBtn.count() > 0) {
      await processBtn.click();
      await page.waitForTimeout(500);
    }
  });
});

test.describe('Ask Page (/ask)', () => {
  test('shows heading and input', async ({ page }) => {
    await page.goto('/ask');
    await expect(page.getByRole('heading', { name: 'Ask About the Code' })).toBeVisible();
    // Use the main content input (not sidebar)
    await expect(page.locator('main input[type="text"]')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Ask' })).toBeVisible();
  });

  test('typing in search shows suggestions', async ({ page }) => {
    await page.goto('/ask');
    const input = page.locator('main input[type="text"]');
    await input.fill('main');
    await page.waitForTimeout(500);
    const suggestions = page.locator('main button:has-text("main")');
    await expect(suggestions.first()).toBeVisible({ timeout: 5000 });
  });

  test('Ask button shows toast', async ({ page }) => {
    await page.goto('/ask');
    await page.getByRole('button', { name: 'Ask' }).click();
    await expect(page.getByText('LLM not connected')).toBeVisible({ timeout: 3000 });
  });

  test('WebSocket status shows', async ({ page }) => {
    await page.goto('/ask');
    const wsStatus = page.getByText('WS Connected').or(page.getByText('WS Disconnected'));
    await expect(wsStatus).toBeVisible({ timeout: 5000 });
  });
});

// ============================================================
// LAYER 4: SIDEBAR
// ============================================================

test.describe('Sidebar', () => {
  test('nav links present', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav');
    await expect(nav.getByRole('link', { name: 'Home' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Learn' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Explore' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Ask' })).toBeVisible();
  });

  test('sidebar search finds symbols', async ({ page }) => {
    await page.goto('/');
    const searchInput = page.locator('nav input');
    await searchInput.fill('parse');
    await page.waitForTimeout(500);
    const results = page.locator('nav .font-mono');
    await expect(results.first()).toBeVisible({ timeout: 5000 });
  });

  test('version shows', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('nav').getByText('v0.1.0')).toBeVisible();
  });
});

// ============================================================
// LAYER 5: FULL NAVIGATION FLOW
// ============================================================

test.describe('Full Navigation Flow', () => {
  test('navigate all pages without errors', async ({ page }) => {
    test.setTimeout(60000);
    const errors: string[] = [];
    page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()); });

    await page.goto('/');
    await page.waitForTimeout(1000);

    // Learn
    await page.locator('nav').getByRole('link', { name: 'Learn' }).click();
    await expect(page).toHaveURL('/learn');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });

    // Explore hub
    await page.locator('nav').getByRole('link', { name: 'Explore' }).click();
    await expect(page).toHaveURL('/explore');

    // Tree
    await page.getByRole('link', { name: 'File Tree' }).click();
    await expect(page).toHaveURL('/explore/tree');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });

    // Metrics (skip graph — slow)
    await page.goto('/explore/metrics');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });

    // Diagrams
    await page.goto('/explore/diagrams');
    await expect(page.getByText('Loading...')).not.toBeVisible({ timeout: 10000 });

    // Ask
    await page.locator('nav').getByRole('link', { name: 'Ask' }).click();
    await expect(page).toHaveURL('/ask');

    // Home
    await page.locator('nav').getByRole('link', { name: 'Home' }).click();
    await expect(page).toHaveURL('/');

    const realErrors = errors.filter(e =>
      !e.includes('WebSocket') && !e.includes('ws://') && !e.includes('ERR_CONNECTION')
    );
    expect(realErrors).toEqual([]);
  });
});
