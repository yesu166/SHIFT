export interface RadialPoint {
  x: number;
  y: number;
}

/**
 * Return a percentage coordinate on a radial action ring.
 * Index zero starts at 12 o'clock and later items move clockwise.
 */
export function getRadialPosition(index: number, count: number, radiusPercent = 36): RadialPoint {
  if (!Number.isInteger(count) || count < 1) {
    throw new RangeError("A radial layout requires at least one item.");
  }
  if (!Number.isInteger(index) || index < 0 || index >= count) {
    throw new RangeError("The radial item index is outside the layout.");
  }
  if (!Number.isFinite(radiusPercent) || radiusPercent <= 0 || radiusPercent > 45) {
    throw new RangeError("The radial radius must be greater than 0 and no more than 45 percent.");
  }

  const angle = -Math.PI / 2 + (2 * Math.PI * index) / count;
  return {
    x: Number((50 + Math.cos(angle) * radiusPercent).toFixed(6)),
    y: Number((50 + Math.sin(angle) * radiusPercent).toFixed(6)),
  };
}
