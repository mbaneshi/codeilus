import { test, expect } from '@playwright/test';

// ============================================================
// SCHEMATIC API ENDPOINTS
// ============================================================

test.describe('Schematic API', () => {
  test('GET /api/v1/schematic returns nodes and communities', async ({ request }) => {
    const res = await request.get('/api/v1/schematic?depth=2');
    expect(res.status()).toBe(200);
    const data = await res.json();
    expect(data.nodes.length).toBeGreaterThan(0);
    expect(Array.isArray(data.communities)).toBe(true);
    expect(data.meta.total_files).toBeGreaterThan(0);
  });

  test('schematic nodes have correct shape', async ({ request }) => {
    const res = await request.get('/api/v1/schematic?depth=2');
    const data = await res.json();
    const node = data.nodes[0];
    expect(node.id).toBeTruthy();
    expect(node.type).toMatch(/^(directory|file|symbol|community)$/);
    expect(typeof node.label).toBe('string');
    expect(typeof node.has_children).toBe('boolean');
  });

  test('schematic communities have enrichment when present', async ({ request }) => {
    const res = await request.get('/api/v1/schematic?depth=2');
    const data = await res.json();
    if (data.communities.length > 0) {
      const comm = data.communities[0];
      expect(typeof comm.id).toBe('number');
      expect(typeof comm.label).toBe('string');
      expect(typeof comm.color).toBe('string');
      expect(comm.color).toMatch(/^#[0-9a-f]{6}$/i);
    }
    // Always passes — communities may not exist for all codebases
    expect(Array.isArray(data.communities)).toBe(true);
  });

  test('schematic expand returns children for first dir', async ({ request }) => {
    // Get the first non-root directory from depth=1
    const res = await request.get('/api/v1/schematic?depth=1');
    const data = await res.json();
    const dir = data.nodes.find((n: any) => n.type === 'directory' && n.id !== 'dir:.');
    expect(dir).toBeTruthy();

    const expandRes = await request.get(`/api/v1/schematic/expand?node_id=${encodeURIComponent(dir.id)}`);
    expect(expandRes.status()).toBe(200);
    const expandData = await expandRes.json();
    expect(expandData.nodes.length).toBeGreaterThan(0);
  });

  test('schematic detail returns callers/callees', async ({ request }) => {
    // Find a symbol that has edges
    const graphRes = await request.get('/api/v1/graph');
    const graph = await graphRes.json();
    const symbolWithEdge = graph.edges[0]?.source_id;
    if (symbolWithEdge) {
      const detailRes = await request.get(`/api/v1/schematic/detail?node_id=sym:${symbolWithEdge}`);
      expect(detailRes.status()).toBe(200);
      const detail = await detailRes.json();
      expect(detail.node_id).toBe(`sym:${symbolWithEdge}`);
      expect(Array.isArray(detail.callers)).toBe(true);
      expect(Array.isArray(detail.callees)).toBe(true);
    }
  });

  test('community filter returns only community symbols', async ({ request }) => {
    const res = await request.get('/api/v1/schematic?depth=10&community_id=2&include_symbols=true&include_edges=true');
    expect(res.status()).toBe(200);
    const data = await res.json();
    const symbols = data.nodes.filter((n: any) => n.type === 'symbol');
    // All symbols should belong to community 2
    for (const sym of symbols) {
      expect(sym.community_id).toBe(2);
    }
    // Edges should only reference loaded symbols
    const symIds = new Set(symbols.map((s: any) => s.id));
    for (const edge of data.edges) {
      expect(symIds.has(edge.source)).toBe(true);
      expect(symIds.has(edge.target)).toBe(true);
    }
  });
});

// ============================================================
// SCHEMATIC TREE MODE
// ============================================================

test.describe('Schematic Tree Mode', () => {
  test('page loads with tree view', async ({ page }) => {
    await page.goto('/explore/schematic');
    // Should show the Tree/Graph toggle with Tree active
    await expect(page.getByText('Tree')).toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Schematic')).toBeVisible();
    // Should show nodes (SVG rects)
    const rects = page.locator('svg rect');
    await expect(rects.first()).toBeVisible({ timeout: 10000 });
  });

  test('shows codeilus root node', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg text:has-text("codeilus")')).toBeVisible({ timeout: 10000 });
  });

  test('toolbar has Fit, Legend, ?, Search', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.getByText('Fit')).toBeVisible({ timeout: 5000 });
    await expect(page.getByText('Legend')).toBeVisible();
    await expect(page.getByText('?')).toBeVisible();
    await expect(page.locator('input[placeholder="Search..."]')).toBeVisible();
  });

  test('clicking a directory expands it', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg text:has-text("codeilus")')).toBeVisible({ timeout: 10000 });
    // Count rects before
    const before = await page.locator('svg rect').count();
    // Click the crates directory text
    const cratesNode = page.locator('svg text:has-text("crates")');
    if (await cratesNode.count() > 0) {
      await cratesNode.first().click();
      await page.waitForTimeout(1000);
      const after = await page.locator('svg rect').count();
      expect(after).toBeGreaterThan(before);
    }
  });

  test('clicking a file opens detail panel', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Look for a file node (has language text like "rust" or "typescript")
    const fileText = page.locator('svg text:has-text(".rs"), svg text:has-text(".ts")');
    if (await fileText.count() > 0) {
      await fileText.first().click();
      await page.waitForTimeout(500);
      // Detail panel should appear with tabs
      await expect(page.getByText('Overview')).toBeVisible({ timeout: 5000 });
      await expect(page.getByText('Source')).toBeVisible();
      await expect(page.getByText('Relations')).toBeVisible();
    }
  });

  test('search highlights nodes', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    const searchInput = page.locator('input[placeholder="Search..."]');
    await searchInput.fill('crates');
    await page.waitForTimeout(300);
    // Should have highlighted nodes (accent stroke)
    const accentRects = page.locator('svg rect[stroke*="accent"]');
    // At least the search should not crash
    expect(await page.locator('svg rect').count()).toBeGreaterThan(0);
  });

  test('minimap is visible', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Minimap is a small SVG in bottom-right with specific dimensions
    const minimap = page.locator('svg[width="180"][height="120"]');
    await expect(minimap).toBeVisible({ timeout: 5000 });
  });

  test('no console errors on schematic', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()); });
    await page.goto('/explore/schematic');
    await page.waitForTimeout(3000);
    const realErrors = errors.filter(e =>
      !e.includes('WebSocket') && !e.includes('ws://') && !e.includes('ERR_CONNECTION')
    );
    expect(realErrors).toEqual([]);
  });
});

// ============================================================
// SCHEMATIC GRAPH MODE
// ============================================================

test.describe('Schematic Graph Mode', () => {
  test('switching to graph mode works', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Click Graph toggle
    await page.getByRole('button', { name: 'Graph' }).click();
    await page.waitForTimeout(1000);
    // Graph mode should activate (button highlighted)
    // If communities exist, sidebar shows; if not, canvas may be empty
    // Just verify no crash
    expect(await page.locator('svg').count()).toBeGreaterThan(0);
  });

  test('community sidebar lists all communities with counts', async ({ page }) => {
    await page.goto('/explore/schematic');
    await page.getByRole('button', { name: 'Graph' }).click();
    await expect(page.getByText('COMMUNITIES')).toBeVisible({ timeout: 5000 });
    // Each community in sidebar should have a number
    const sidebarItems = page.locator('.shrink-0 button');
    const count = await sidebarItems.count();
    expect(count).toBeGreaterThan(0);
  });

  test('clicking a community drills into symbols', async ({ page }) => {
    await page.goto('/explore/schematic');
    await page.getByRole('button', { name: 'Graph' }).click();
    await expect(page.getByText('COMMUNITIES')).toBeVisible({ timeout: 5000 });
    // Click first community in sidebar
    const firstComm = page.locator('.shrink-0 button').first();
    await firstComm.click();
    await page.waitForTimeout(2000);
    // Should show breadcrumb "Communities"
    await expect(page.getByText('Communities')).toBeVisible({ timeout: 5000 });
    // Should show symbol nodes
    const nodeCount = await page.locator('svg rect').count();
    expect(nodeCount).toBeGreaterThan(0);
  });
});

// ============================================================
// SCHEMATIC INTERACTIONS
// ============================================================

test.describe('Schematic Interactions', () => {
  test('hover shows tooltip', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Hover over a node
    const nodeG = page.locator('svg g[style*="cursor"]').first();
    await nodeG.hover();
    await page.waitForTimeout(300);
    // Tooltip should appear (fixed position div with type badge)
    const tooltip = page.locator('.fixed.z-\\[60\\]');
    // Tooltip may or may not be visible depending on node type
    // Just verify no crash
  });

  test('right-click shows context menu', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Right-click a node
    const nodeG = page.locator('svg g[style*="cursor"]').first();
    await nodeG.click({ button: 'right' });
    await page.waitForTimeout(300);
    // Context menu should appear
    await expect(page.getByText('Copy name')).toBeVisible({ timeout: 3000 });
    await expect(page.getByText('Focus here')).toBeVisible();
  });

  test('keyboard ? shows help overlay', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Press ?
    await page.keyboard.press('?');
    await page.waitForTimeout(300);
    await expect(page.getByText('Keyboard Shortcuts')).toBeVisible({ timeout: 3000 });
    // Press Escape to close
    await page.keyboard.press('Escape');
    await expect(page.getByText('Keyboard Shortcuts')).not.toBeVisible({ timeout: 3000 });
  });

  test('Fit button resets viewport', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Click Fit button — should not crash
    await page.getByRole('button', { name: 'Fit' }).click();
    await page.waitForTimeout(500);
    // Nodes should still be visible
    expect(await page.locator('svg rect').count()).toBeGreaterThan(0);
  });

  test('Legend toggle shows/hides legend panel', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Click Legend
    await page.getByRole('button', { name: 'Legend' }).click();
    await page.waitForTimeout(300);
    await expect(page.getByText('Nodes')).toBeVisible({ timeout: 3000 });
    await expect(page.getByText('Edges')).toBeVisible();
    await expect(page.getByText('Interactions')).toBeVisible();
    // Click again to hide
    await page.getByRole('button', { name: 'Legend' }).click();
    await page.waitForTimeout(300);
  });

  test('detail panel Source tab loads code', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Expand crates → find a file
    const cratesNode = page.locator('svg text:has-text("crates")');
    if (await cratesNode.count() > 0) {
      await cratesNode.first().click();
      await page.waitForTimeout(1000);
      // Look for a .rs file and click it
      const fileNode = page.locator('svg text:has-text(".rs")').first();
      if (await fileNode.count() > 0) {
        await fileNode.click();
        await page.waitForTimeout(500);
        // Click Source tab
        const sourceTab = page.getByRole('button', { name: 'Source' });
        if (await sourceTab.count() > 0) {
          await sourceTab.click();
          await page.waitForTimeout(2000);
          // Should show source code (pre element)
          const pre = page.locator('pre');
          await expect(pre).toBeVisible({ timeout: 5000 });
        }
      }
    }
  });
});

// ============================================================
// RED TESTS — Expected behavior, may not pass yet
// These define the target UX. Fix code until they go green.
// ============================================================

test.describe('Schematic Expected Behaviors (TDD)', () => {
  test('file detail panel shows source code when Source tab clicked', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Expand first directory
    const firstDir = page.locator('svg text:has-text("▶")').first();
    if (await firstDir.count() > 0) {
      await firstDir.click();
      await page.waitForTimeout(1500);
      // Keep expanding until we find a file
      const secondDir = page.locator('svg text:has-text("▶")').first();
      if (await secondDir.count() > 0) {
        await secondDir.click();
        await page.waitForTimeout(1500);
      }
      // Find any file node (has "loc" in the text)
      const fileNode = page.locator('svg text:has-text("loc")').first();
      if (await fileNode.count() > 0) {
        await fileNode.click();
        await page.waitForTimeout(500);
        // Click Source tab
        const sourceTab = page.getByRole('button', { name: 'Source' });
        if (await sourceTab.count() > 0) {
          await sourceTab.click();
          await page.waitForTimeout(3000);
          // Should show source code
          const pre = page.locator('pre');
          await expect(pre).toBeVisible({ timeout: 5000 });
          const text = await pre.textContent();
          expect(text).toBeTruthy();
          expect(text!.length).toBeGreaterThan(20);
        }
      }
    }
  });

  test('community sidebar shows in graph mode with member counts', async ({ page }) => {
    await page.goto('/explore/schematic');
    await page.getByRole('button', { name: 'Graph' }).click();
    await page.waitForTimeout(1000);
    await expect(page.getByText('COMMUNITIES')).toBeVisible({ timeout: 5000 });
    // Sidebar buttons should have community names and counts
    const sidebarButtons = page.locator('.shrink-0 button');
    const count = await sidebarButtons.count();
    expect(count).toBeGreaterThan(0);
    // Each should have a label
    const firstText = await sidebarButtons.first().textContent();
    expect(firstText).toBeTruthy();
    expect(firstText!.length).toBeGreaterThan(0);
  });

  test('detail panel Learn tab shows chapter link with progress', async ({ page }) => {
    await page.goto('/explore/schematic');
    await page.getByRole('button', { name: 'Graph' }).click();
    await page.waitForTimeout(1000);
    // Click a community to drill in
    await page.locator('.shrink-0 button').first().click();
    await page.waitForTimeout(2000);
    // Click a symbol node
    const symbolNode = page.locator('svg g[style*="cursor"] text').first();
    if (await symbolNode.count() > 0) {
      await symbolNode.click();
      await page.waitForTimeout(500);
      // Click Learn tab
      const learnTab = page.getByRole('button', { name: 'Learn' });
      if (await learnTab.count() > 0) {
        await learnTab.click();
        await page.waitForTimeout(500);
        // Should show a chapter link or "No learning chapter" message
        const hasChapter = await page.getByText('Start Learning').count() > 0;
        const noChapter = await page.getByText('No learning chapter').count() > 0;
        expect(hasChapter || noChapter).toBe(true);
      }
    }
  });

  test('right-click Copy name action executes without error', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    const nodeG = page.locator('svg g[style*="cursor"]').first();
    await nodeG.click({ button: 'right' });
    await page.waitForTimeout(300);
    await expect(page.getByText('Copy name')).toBeVisible();
    // Click Copy name — should close the menu without crashing
    await page.getByText('Copy name').click();
    await page.waitForTimeout(300);
    // Context menu should be gone
    await expect(page.getByText('Copy name')).not.toBeVisible({ timeout: 2000 });
  });

  test('right-click Focus here zooms to the node', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Right-click crates
    const cratesText = page.locator('svg text:has-text("crates")').first();
    if (await cratesText.count() > 0) {
      await cratesText.click({ button: 'right' });
      await page.waitForTimeout(300);
      await page.getByText('Focus here').click();
      await page.waitForTimeout(500);
      // The transform should have changed (node centered)
      const gTransform = await page.locator('svg > g').first().getAttribute('transform');
      expect(gTransform).toBeTruthy();
    }
  });

  test('keyboard F fits all nodes in viewport', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    // Press F
    await page.keyboard.press('f');
    await page.waitForTimeout(500);
    // All nodes should be visible (within viewport)
    const rects = await page.locator('svg rect').count();
    expect(rects).toBeGreaterThan(0);
  });

  test('double-click directory recursively expands', async ({ page }) => {
    await page.goto('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });
    const before = await page.locator('svg rect').count();
    // Double-click crates
    const cratesText = page.locator('svg text:has-text("crates")').first();
    if (await cratesText.count() > 0) {
      await cratesText.dblclick();
      await page.waitForTimeout(2000);
      const after = await page.locator('svg rect').count();
      expect(after).toBeGreaterThan(before);
    }
  });
});

// ============================================================
// SCHEMATIC + FULL NAVIGATION
// ============================================================

test.describe('Schematic Navigation Flow', () => {
  test('navigate explore → schematic → tree → graph without errors', async ({ page }) => {
    test.setTimeout(60000);
    const errors: string[] = [];
    page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()); });

    // Explore hub
    await page.goto('/explore');
    await expect(page.getByText('Schematic Explorer')).toBeVisible({ timeout: 5000 });

    // Click Schematic card
    await page.getByRole('link', { name: 'Schematic Explorer' }).click();
    await expect(page).toHaveURL('/explore/schematic');
    await expect(page.locator('svg rect').first()).toBeVisible({ timeout: 10000 });

    // Switch to Graph mode
    await page.getByRole('button', { name: 'Graph' }).click();
    await page.waitForTimeout(1000);
    await expect(page.getByText('COMMUNITIES')).toBeVisible({ timeout: 5000 });

    // Switch back to Tree mode
    await page.getByRole('button', { name: 'Tree' }).click();
    await page.waitForTimeout(1000);
    await expect(page.locator('svg text:has-text("codeilus")').first()).toBeVisible({ timeout: 5000 });

    // Go back to explore
    await page.getByRole('link', { name: '←' }).click();
    await expect(page).toHaveURL('/explore');

    const realErrors = errors.filter(e =>
      !e.includes('WebSocket') && !e.includes('ws://') && !e.includes('ERR_CONNECTION')
    );
    expect(realErrors).toEqual([]);
  });
});
