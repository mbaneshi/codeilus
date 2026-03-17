<script lang="ts">
  import { onMount } from 'svelte';

  let {
    width = 4000,
    height = 3000,
    children,
  }: {
    width?: number;
    height?: number;
    children?: import('svelte').Snippet;
  } = $props();

  let svgEl: SVGSVGElement | undefined = $state();
  let tx = $state(50);
  let ty = $state(50);
  let scale = $state(0.8);
  let dragging = $state(false);
  let dragStart = { x: 0, y: 0, tx: 0, ty: 0 };

  function onWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.92 : 1.08;
    const newScale = Math.max(0.05, Math.min(4, scale * factor));
    // Zoom towards cursor
    const rect = svgEl!.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    tx = mx - (mx - tx) * (newScale / scale);
    ty = my - (my - ty) * (newScale / scale);
    scale = newScale;
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    dragging = true;
    dragStart = { x: e.clientX, y: e.clientY, tx, ty };
    (e.target as Element).setPointerCapture?.(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    tx = dragStart.tx + (e.clientX - dragStart.x);
    ty = dragStart.ty + (e.clientY - dragStart.y);
  }

  function onPointerUp() {
    dragging = false;
  }

  export function fitToView(contentWidth: number, contentHeight: number) {
    if (!svgEl) return;
    const rect = svgEl.getBoundingClientRect();
    const padX = 80, padY = 80;
    const scaleX = (rect.width - padX) / contentWidth;
    const scaleY = (rect.height - padY) / contentHeight;
    scale = Math.min(scaleX, scaleY, 1.5);
    tx = (rect.width - contentWidth * scale) / 2;
    ty = (rect.height - contentHeight * scale) / 2;
  }

  export function zoomToNode(x: number, y: number, w: number, h: number) {
    if (!svgEl) return;
    const rect = svgEl.getBoundingClientRect();
    const targetScale = Math.min(rect.width / (w + 200), rect.height / (h + 200), 1.5);
    scale = targetScale;
    tx = rect.width / 2 - (x + w / 2) * scale;
    ty = rect.height / 2 - (y + h / 2) * scale;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<svg
  bind:this={svgEl}
  class="w-full h-full select-none"
  style="cursor: {dragging ? 'grabbing' : 'grab'}; background: var(--surface-0);"
  onwheel={onWheel}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
>
  <defs>
    <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
      <polygon points="0 0, 8 3, 0 6" fill="var(--c-text-muted)" />
    </marker>
  </defs>
  <g transform="translate({tx},{ty}) scale({scale})">
    {#if children}
      {@render children()}
    {/if}
  </g>
</svg>
