export type OperationStatus = "available" | "planned";
export type OperationCategory = "compress" | "convert" | "organize" | "edit" | "protect" | "action";
export type WheelMode = "convert" | "tools" | "actions";

export interface OperationDefinition {
  id: string;
  title: string;
  description: string;
  category: OperationCategory;
  acceptedExtensions: readonly string[];
  outputLabel: string;
  status: OperationStatus;
  wheelMode?: WheelMode;
  targetFormat?: "png" | "jpg" | "webp" | "bmp" | "tiff";
  unavailableReason?: string;
}

const IMAGE_INPUTS = [".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tif", ".tiff"] as const;

export const OPERATIONS: readonly OperationDefinition[] = [
  {
    id: "create-zip", title: "Create ZIP", description: "Bundle files and folders into a validated DEFLATE archive.",
    category: "compress", acceptedExtensions: ["*"], outputLabel: "ZIP archive", status: "available", wheelMode: "tools",
  },
  {
    id: "compress-jpeg", title: "Compress JPG", description: "Re-encode JPEG images locally at quality 78; preserve the original.",
    category: "compress", acceptedExtensions: [".jpg", ".jpeg"], outputLabel: "JPEG image", status: "available", wheelMode: "tools",
  },
  {
    id: "convert-png", title: "PNG", description: "Convert supported image files to PNG.",
    category: "convert", acceptedExtensions: IMAGE_INPUTS, outputLabel: "PNG image", status: "available", wheelMode: "convert", targetFormat: "png",
  },
  {
    id: "convert-jpg", title: "JPG", description: "Convert supported image files to JPEG.",
    category: "convert", acceptedExtensions: IMAGE_INPUTS, outputLabel: "JPEG image", status: "available", wheelMode: "convert", targetFormat: "jpg",
  },
  {
    id: "convert-webp", title: "WebP", description: "Convert supported image files to WebP.",
    category: "convert", acceptedExtensions: IMAGE_INPUTS, outputLabel: "WebP image", status: "available", wheelMode: "convert", targetFormat: "webp",
  },
  {
    id: "convert-bmp", title: "BMP", description: "Convert supported image files to BMP.",
    category: "convert", acceptedExtensions: IMAGE_INPUTS, outputLabel: "BMP image", status: "available", wheelMode: "convert", targetFormat: "bmp",
  },
  {
    id: "convert-tiff", title: "TIFF", description: "Convert supported image files to TIFF.",
    category: "convert", acceptedExtensions: IMAGE_INPUTS, outputLabel: "TIFF image", status: "available", wheelMode: "convert", targetFormat: "tiff",
  },
  {
    id: "copy-paths", title: "Copy paths", description: "Copy selected local file paths as newline-separated text.",
    category: "action", acceptedExtensions: ["*"], outputLabel: "Clipboard text", status: "available", wheelMode: "actions",
  },
  {
    id: "open-folder", title: "Output folder", description: "Open the SHIFT output destination.",
    category: "action", acceptedExtensions: ["*"], outputLabel: "Folder", status: "available", wheelMode: "actions",
  },
  {
    id: "optimize-pdf", title: "Optimize PDF", description: "Reduce PDF size while preserving the original.",
    category: "compress", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "planned",
    unavailableReason: "A verified PDF optimization engine is not connected yet.",
  },
  {
    id: "merge-pdf", title: "Merge PDFs", description: "Combine selected PDFs in their current order.",
    category: "organize", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "planned",
    unavailableReason: "PDF structural operations are not implemented yet.",
  },
  {
    id: "split-pdf", title: "Split PDF", description: "Extract a page range into a new PDF.",
    category: "organize", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "planned",
    unavailableReason: "PDF structural operations are not implemented yet.",
  },
  {
    id: "docx-to-pdf", title: "Word to PDF", description: "Convert a Word document to PDF.",
    category: "convert", acceptedExtensions: [".docx"], outputLabel: "PDF", status: "planned",
    unavailableReason: "Office conversion and fidelity validation are not connected yet.",
  },
  {
    id: "xlsx-to-pdf", title: "Excel to PDF", description: "Convert an Excel workbook to PDF.",
    category: "convert", acceptedExtensions: [".xlsx"], outputLabel: "PDF", status: "planned",
    unavailableReason: "Office conversion and fidelity validation are not connected yet.",
  },
  {
    id: "pptx-to-pdf", title: "PowerPoint to PDF", description: "Convert a presentation to PDF.",
    category: "convert", acceptedExtensions: [".pptx"], outputLabel: "PDF", status: "planned",
    unavailableReason: "Office conversion and fidelity validation are not connected yet.",
  },
  {
    id: "watermark-pdf", title: "Add PDF watermark", description: "Write a text watermark to PDF pages.",
    category: "edit", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "planned",
    unavailableReason: "A verified PDF-writing implementation has not been selected.",
  },
  {
    id: "protect-pdf", title: "Password-protect PDF", description: "Encrypt a PDF with a supplied password.",
    category: "protect", acceptedExtensions: [".pdf"], outputLabel: "Protected PDF", status: "planned",
    unavailableReason: "PDF encryption is not implemented yet.",
  },
];

export function extensionOf(filename: string): string {
  const leaf = filename.split(/[\\/]/).pop() ?? filename;
  const dot = leaf.lastIndexOf(".");
  return dot <= 0 ? "" : leaf.slice(dot).toLowerCase();
}

function targetAlreadyPresent(target: OperationDefinition["targetFormat"], extensions: readonly string[]): boolean {
  if (!target) return false;
  if (target === "jpg") return extensions.some((ext) => ext === ".jpg" || ext === ".jpeg");
  if (target === "tiff") return extensions.some((ext) => ext === ".tif" || ext === ".tiff");
  return extensions.includes("." + target);
}

export function getAvailableOperations(fileNames: readonly string[]): OperationDefinition[] {
  if (fileNames.length === 0) return [];
  const extensions = fileNames.map(extensionOf);
  return OPERATIONS.filter((operation) =>
    operation.status === "available"
    && (operation.acceptedExtensions.includes("*")
      || extensions.every((extension) => operation.acceptedExtensions.includes(extension)))
    && !targetAlreadyPresent(operation.targetFormat, extensions),
  );
}

export function getWheelOperations(fileNames: readonly string[], mode: WheelMode): OperationDefinition[] {
  return getAvailableOperations(fileNames).filter((operation) => operation.wheelMode === mode);
}

export function getOperationById(id: string): OperationDefinition | undefined {
  return OPERATIONS.find((operation) => operation.id === id);
}
