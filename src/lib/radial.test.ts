import { describe, expect, it } from "vitest";
import { getRadialPosition } from "./radial";

describe("getRadialPosition", () => {
  it("places the first action at the top of the ring", () => {
    expect(getRadialPosition(0, 1)).toEqual({ x: 50, y: 14 });
  });

  it("spreads two actions opposite each other", () => {
    expect(getRadialPosition(0, 2)).toEqual({ x: 50, y: 14 });
    expect(getRadialPosition(1, 2)).toEqual({ x: 50, y: 86 });
  });

  it("starts at twelve o'clock and rotates clockwise", () => {
    const north = getRadialPosition(0, 4);
    const east = getRadialPosition(1, 4);
    const south = getRadialPosition(2, 4);
    const west = getRadialPosition(3, 4);
    expect(north.y).toBeLessThan(50);
    expect(east.x).toBeGreaterThan(50);
    expect(south.y).toBeGreaterThan(50);
    expect(west.x).toBeLessThan(50);
  });

  it("rejects empty layouts and invalid indices", () => {
    expect(() => getRadialPosition(0, 0)).toThrow(RangeError);
    expect(() => getRadialPosition(-1, 4)).toThrow(RangeError);
    expect(() => getRadialPosition(4, 4)).toThrow(RangeError);
  });

  it("guards against radii that could place menu items off the ring", () => {
    expect(() => getRadialPosition(0, 2, 46)).toThrow(RangeError);
    expect(() => getRadialPosition(0, 2, Number.NaN)).toThrow(RangeError);
  });
});
