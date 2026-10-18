import { describe, expect, it } from "vitest";
import { formatBytes, formatSignedBytes } from "./format";

describe("formatBytes", () => {
  it("handles zero and invalid byte counts", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(-1)).toBe("—");
    expect(formatBytes(Number.NaN)).toBe("—");
  });

  it("formats base-10 units", () => {
    expect(formatBytes(1_000)).toBe("1 KB");
    expect(formatBytes(1_200_000)).toBe("1.2 MB");
  });
});

describe("formatSignedBytes", () => {
  it("distinguishes savings, growth, and equal size", () => {
    expect(formatSignedBytes(900)).toBe("900 B saved");
    expect(formatSignedBytes(-900)).toBe("900 B larger");
    expect(formatSignedBytes(0)).toBe("No size change");
  });
});
