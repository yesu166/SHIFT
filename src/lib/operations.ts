export type OperationStatus = "available" | "planned";
export type OperationCategory = "compress" | "convert" | "organize" | "edit" | "protect";

export interface OperationDefinition {
  id: string;
  title: string;
  description: string;
  category: OperationCategory;
  acceptedExtensions: readonly string[];
  outputLabel: string;
  status: OperationStatus;
  unavailableReason?: string;
}

export const OPERATIONS: readonly OperationDefinition[] = [
  {
    id: "create-zip", title: "Compress files to ZIP",
    description: "Create a DEFLATE archive and verify every entry.",
    category: "compress", acceptedExtensions: ["*"], outputLabel: "ZIP archive", status: "available",
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

export function getAvailableOperations(fileNames: readonly string[]): OperationDefinition[] {
  if (fileNames.length === 0) return [];
  const extensions = fileNames.map(extensionOf);
  return OPERATIONS.filter((operation) =>
    operation.status === "available"
    && (operation.acceptedExtensions.includes("*")
      || extensions.every((extension) => operation.acceptedExtensions.includes(extension))),
  );
}

export function getOperationById(id: string): OperationDefinition | undefined {
  return OPERATIONS.find((operation) => operation.id === id);
}
