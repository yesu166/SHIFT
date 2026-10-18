import { describe, expect, it } from "vitest";
import { extensionOf, getAvailableOperations, getOperationById, OPERATIONS } from "./operations";

describe("SHIFT operation registry", () => {
  it("offers no action without selected inputs", () => {
    expect(getAvailableOperations([])).toEqual([]);
  });

  it("offers ZIP creation for a mixed selection", () => {
    expect(getAvailableOperations(["report.pdf", "slides.pptx", "notes.txt"]).map((op) => op.id))
      .toEqual(["create-zip"]);
  });

  it("never offers planned actions as executable", () => {
    expect(getAvailableOperations(["report.pdf"]).every((op) => op.status === "available")).toBe(true);
    expect(getAvailableOperations(["report.pdf"]).map((op) => op.id)).not.toContain("merge-pdf");
  });

  it("detects extensions case-insensitively and handles Windows paths", () => {
    expect(extensionOf("C:\\\\Users\\\\Yesu\\\\Report.PDF")).toBe(".pdf");
    expect(extensionOf(".env")).toBe("");
    expect(extensionOf("extensionless")).toBe("");
  });

  it("has unique stable IDs and one connected first-milestone operation", () => {
    const ids = OPERATIONS.map((op) => op.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(OPERATIONS.filter((op) => op.status === "available").map((op) => op.id)).toEqual(["create-zip"]);
    expect(getOperationById("create-zip")?.outputLabel).toBe("ZIP archive");
  });
});
