import { describe, expect, it } from "vitest";
import {
  extensionOf, getAvailableOperations, getOperationById, getUnavailableWheelOperations, getWheelOperations,
  NO_OPTIONAL_ENGINES, OPERATIONS, type EngineAvailability,
} from "./operations";

const installed: EngineAvailability = {
  ...NO_OPTIONAL_ENGINES,
  ffmpeg: true,
  libreOffice: true,
  qpdf: true,
  poppler: true,
  sevenZip: true,
  inkscape: true,
};

describe("SHIFT operation registry", () => {
  it("offers no action without selected inputs", () => {
    expect(getAvailableOperations([])).toEqual([]);
  });

  it("offers ZIP/TAR/GZIP tools and avoids GZIP for a multi-file selection", () => {
    const ids = getWheelOperations(["report.pdf", "slides.pptx", "notes.txt"], "tools").map((op) => op.id);
    expect(ids).toContain("create-zip");
    expect(ids).toContain("create-tar");
    expect(ids).toContain("create-targz");
    expect(ids).not.toContain("create-gzip");
  });

  it("offers compatible built-in image targets while hiding the current format", () => {
    expect(getWheelOperations(["photo.jpg"], "convert").map((op) => op.targetFormat))
      .toContain("webp");
    expect(getWheelOperations(["photo.jpg"], "convert").map((op) => op.targetFormat))
      .not.toContain("jpg");
    expect(getWheelOperations(["a.jpg", "b.png"], "convert").map((op) => op.targetFormat))
      .not.toContain("png");
  });

  it("gates FFmpeg conversions on engine detection", () => {
    expect(getWheelOperations(["recording.mp4"], "convert")).toEqual([]);
    expect(getWheelOperations(["recording.mp4"], "convert", installed).map((op) => op.id))
      .toContain("media-video-mkv");
    expect(getWheelOperations(["recording.mp4"], "convert", installed).map((op) => op.id))
      .toContain("extract-audio-mp3");
  });

  it("explains compatible options that are hidden behind missing engines", () => {
    const pdfOptions = getUnavailableWheelOperations(["report.pdf"], "convert");
    expect(pdfOptions.map((op) => op.id)).toContain("pdf-to-image-jpg");
    expect(pdfOptions.map((op) => op.id)).toContain("pdf-office-docx");
    const imageOptions = getUnavailableWheelOperations(["photo.jpg"], "convert");
    expect(imageOptions.map((op) => op.id)).toContain("ffmpeg-image-convert-avif");
    expect(imageOptions.map((op) => op.id)).toContain("image-to-pdf");
    expect(getUnavailableWheelOperations(["photo.jpg"], "convert", installed)).toEqual([]);
  });

  it("gates Office and PDF tools on their native engines", () => {
    expect(getWheelOperations(["sheet.xlsx"], "convert").map((op) => op.id))
      .not.toContain("spreadsheet-convert-pdf");
    expect(getWheelOperations(["sheet.xlsx"], "convert", installed).map((op) => op.id))
      .toContain("spreadsheet-convert-pdf");
    expect(getWheelOperations(["report.pdf", "appendix.pdf"], "tools", installed).map((op) => op.id))
      .toContain("pdf-merge");
    expect(getWheelOperations(["report.pdf"], "tools", installed).map((op) => op.id))
      .not.toContain("pdf-merge");
  });

  it("gates SVG export on Inkscape", () => {
    expect(getWheelOperations(["icon.svg"], "convert").map((op) => op.id)).toEqual([]);
    expect(getWheelOperations(["icon.svg"], "convert", installed).map((op) => op.id))
      .toEqual(["svg-to-png", "svg-to-pdf"]);
  });

  it("renders Markdown/text to HTML locally without optional engines", () => {
    expect(getWheelOperations(["notes.md"], "convert").map((op) => op.id)).toContain("text-to-html");
  });

  it("converts raster images to PDF only when LibreOffice is detected", () => {
    expect(getWheelOperations(["scan.jpg"], "convert").map((op) => op.id)).not.toContain("image-to-pdf");
    expect(getWheelOperations(["scan.jpg"], "convert", installed).map((op) => op.id)).toContain("image-to-pdf");
  });

  it("requires both LibreOffice and Poppler to render presentation slides as images", () => {
    const libreOfficeOnly = { ...installed, poppler: false };
    expect(getWheelOperations(["deck.pptx"], "convert", libreOfficeOnly).map((op) => op.id))
      .not.toContain("presentation-convert-jpg");
    expect(getWheelOperations(["deck.pptx"], "convert", installed).map((op) => op.id))
      .toContain("presentation-convert-jpg");
    expect(getWheelOperations(["deck.pptx"], "convert", installed).map((op) => op.id))
      .toContain("presentation-convert-pdf");
  });

  it("offers HEIC and AVIF exports only when FFmpeg is available", () => {
    expect(getWheelOperations(["photo.png"], "convert").map((op) => op.id))
      .not.toContain("ffmpeg-image-convert-avif");
    expect(getWheelOperations(["photo.png"], "convert", installed).map((op) => op.id))
      .toContain("ffmpeg-image-convert-avif");
    expect(getWheelOperations(["photo.png"], "convert", installed).map((op) => op.id))
      .toContain("ffmpeg-image-convert-heic");
  });

  it("converts TAR and GZIP inputs to ZIP without optional tools", () => {
    expect(getWheelOperations(["bundle.tar"], "convert").map((op) => op.id)).toContain("tar-to-zip");
    expect(getWheelOperations(["notes.gz"], "convert").map((op) => op.id)).toContain("tar-to-zip");
    expect(getWheelOperations(["backup.rar"], "convert").map((op) => op.id)).not.toContain("archive-to-zip");
    expect(getWheelOperations(["backup.rar"], "convert", installed).map((op) => op.id)).toContain("archive-to-zip");
  });

  it("offers native subtitle conversion without optional engines", () => {
    expect(getWheelOperations(["captions.srt"], "convert").map((op) => op.id))
      .toContain("subtitle-srt-vtt");
    expect(getWheelOperations(["captions.vtt"], "convert").map((op) => op.id))
      .toContain("subtitle-vtt-srt");
  });

  it("offers JPEG compression only for JPEG input", () => {
    expect(getWheelOperations(["photo.jpeg"], "tools").map((op) => op.id))
      .toContain("compress-jpeg");
    expect(getWheelOperations(["photo.png"], "tools").map((op) => op.id))
      .not.toContain("compress-jpeg");
  });

  it("never offers operations marked planned as executable", () => {
    expect(getAvailableOperations(["report.pdf"], installed).every((op) => op.status === "available")).toBe(true);
    expect(getAvailableOperations(["report.pdf"], installed).map((op) => op.id)).not.toContain("planned-pdf-split");
  });

  it("detects extensions case-insensitively and handles Windows paths", () => {
    expect(extensionOf("C:\\Users\\Yesu\\Report.PDF")).toBe(".pdf");
    expect(extensionOf(".env")).toBe("");
    expect(extensionOf("extensionless")).toBe("");
  });

  it("has unique stable IDs and registers every visible format family", () => {
    const ids = OPERATIONS.map((op) => op.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const id of [
      "convert-png", "media-audio-mp3", "media-video-mp4", "pdf-optimize",
      "office-convert-docx", "spreadsheet-convert-csv", "subtitle-srt-vtt",
      "create-zip", "create-tar", "create-targz", "create-gzip",
    ]) expect(getOperationById(id)).toBeDefined();
  });
});
