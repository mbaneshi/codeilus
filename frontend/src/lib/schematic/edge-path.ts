/** Convert ELK edge bend points to an SVG path d-attribute. */
export function edgePathD(points: { x: number; y: number }[]): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;
  if (points.length === 2) {
    return `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y}`;
  }

  // Smooth cubic bezier through points
  let d = `M ${points[0].x} ${points[0].y}`;
  for (let i = 1; i < points.length - 1; i++) {
    const prev = points[i - 1];
    const curr = points[i];
    const next = points[i + 1];
    const cpx1 = (prev.x + curr.x) / 2;
    const cpy1 = (prev.y + curr.y) / 2;
    const cpx2 = (curr.x + next.x) / 2;
    const cpy2 = (curr.y + next.y) / 2;
    if (i === 1) {
      d += ` Q ${curr.x} ${curr.y} ${cpx2} ${cpy2}`;
    } else {
      d += ` T ${cpx2} ${cpy2}`;
    }
  }
  const last = points[points.length - 1];
  d += ` L ${last.x} ${last.y}`;
  return d;
}

/** Get the midpoint of a path for label placement. */
export function edgeMidpoint(points: { x: number; y: number }[]): { x: number; y: number } {
  if (points.length === 0) return { x: 0, y: 0 };
  if (points.length === 1) return points[0];
  const mid = Math.floor(points.length / 2);
  if (points.length % 2 === 0) {
    return {
      x: (points[mid - 1].x + points[mid].x) / 2,
      y: (points[mid - 1].y + points[mid].y) / 2,
    };
  }
  return points[mid];
}
