<script lang="ts">
  import type { LayoutNode } from '$lib/schematic/layout';

  let {
    nodes = [],
    canvasW = 4000,
    canvasH = 3000,
    viewX = 0,
    viewY = 0,
    viewScale = 1,
    viewW = 800,
    viewH = 600,
    onpan,
  }: {
    nodes: LayoutNode[];
    canvasW: number;
    canvasH: number;
    viewX: number;
    viewY: number;
    viewScale: number;
    viewW: number;
    viewH: number;
    onpan: (tx: number, ty: number) => void;
  } = $props();

  const W = 180;
  const H = 120;
  let minimapScale = $derived(Math.min(W / Math.max(canvasW, 1), H / Math.max(canvasH, 1)));

  // Viewport rectangle in minimap coords
  let vpX = $derived(-viewX / viewScale * minimapScale);
  let vpY = $derived(-viewY / viewScale * minimapScale);
  let vpW = $derived(viewW / viewScale * minimapScale);
  let vpH = $derived(viewH / viewScale * minimapScale);

  let dragging = $state(false);

  function handleClick(e: MouseEvent | PointerEvent) {
    const rect = (e.currentTarget as SVGElement).getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    const canvasX = mx / minimapScale;
    const canvasY = my / minimapScale;
    onpan(-canvasX * viewScale + viewW / 2, -canvasY * viewScale + viewH / 2);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<svg
  class="absolute bottom-3 right-3 z-20 rounded-lg border border-[var(--c-border)] shadow-lg"
  width={W} height={H}
  style="background: var(--surface-1); opacity: 0.85;"
  onclick={handleClick}
  onpointermove={(e) => dragging && handleClick(e)}
  onpointerdown={() => dragging = true}
  onpointerup={() => dragging = false}
>
  <!-- Nodes -->
  {#each nodes as node}
    {@const nx = node.x * minimapScale}
    {@const ny = node.y * minimapScale}
    {@const nw = Math.max(node.width * minimapScale, 2)}
    {@const nh = Math.max(node.height * minimapScale, 1.5)}
    <rect x={nx} y={ny} width={nw} height={nh} rx="0.5"
      fill={node.data?.community_color || 'var(--c-text-muted)'}
      opacity="0.6"
    />
  {/each}

  <!-- Viewport rectangle -->
  <rect
    x={vpX} y={vpY} width={Math.max(vpW, 4)} height={Math.max(vpH, 3)}
    fill="none" stroke="var(--c-accent)" stroke-width="1.5" rx="1"
    opacity="0.8"
  />
</svg>
