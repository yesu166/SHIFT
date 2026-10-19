import { describe, expect, it } from "vitest";
import { extensionOf, getAvailableOperations, getOperationById, getWheelOperations, OPERATIONS } from "./operations";

describe("SHIFT operation registry", () => {
  it("offers no action without selected inputs", () => {
    expect(getAvailableOperations([])).toEqual([]);
  });

  it("offers ZIP and safe file actions for arbitrary mixed selections", () => {
    expect(getAvailableOperations(["report.pdf", "slides.pptx", "notes.txt"]).map((op) => op.id))
      .toEqual(["create-zip", "copy-paths", "open-folder"]);
  });

  it("offers compatible image target formats while hiding the current format", () => {
    expect(getWheelOperations(["photo.jpg"], "convert").map((op) => op.targetFormat))
      .toEqual(["png", "webp", "bmp", "tiff"]);
    expect(getWheelOperations(["a.jpg", "b.png"], "convert").map((op) => op.targetFormat))
      .toEqual(["webp", "bmp", "tiff"]);
  });

  it("offers JPEG compression only for JPEG selections", () => {
    expect(getWheelOperations(["photo.jpeg"], "tools").map((op) => op.id))
      .toEqual(["create-zip", "compress-jpeg"]);
    expect(getWheelOperations(["photo.png"], "tools").map((op) => op.id))
      .toEqual(["create-zip"]);
  });

  it("never offers planned actions as executable", () => {
    expect(getAvailableOperations(["report.pdf"]).every((op) => op.status === "available")).toBe(true);
    expect(getAvailableOperations(["report.pdf"]).map((op) => op.id)).not.toContain("merge-pdf");
  });

  it("detects extensions case-insensitively and handles Windows paths", () => {
    expect(extensionOf("C:\\Users\\Yesu\\Report.PDF")).toBe(".pdf");
    expect(extensionOf(".env")).toBe("");
    expect(extensionOf("extensionless")).toBe("");
  });

  it("has unique stable IDs and a connected core operation", () => {
    const ids = OPERATIONS.map((op) => op.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(OPERATIONS.filter((op) => op.status === "available").map((op) => op.id))
      .toEqual(["create-zip", "compress-jpeg", "convert-png", "convert-jpg", "convert-webp", "convert-bmp", "convert-tiff", "copy-paths", "open-folder"]);
    expect(getOperationById("create-zip")?.outputLabel).toBe("ZIP archive");
  });
});
