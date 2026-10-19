export type OperationStatus = "available" | "planned";
export type OperationCategory = "compress" | "convert" | "organize" | "edit" | "protect" | "action";
export type WheelMode = "convert" | "tools" | "actions";
export type RequiredEngine = "ffmpeg" | "libreOffice" | "qpdf" | "poppler" | "sevenZip" | "inkscape";

export interface EngineAvailability {
  ffmpeg: boolean;
  libreOffice: boolean;
  qpdf: boolean;
  poppler: boolean;
  sevenZip: boolean;
  inkscape: boolean;
  builtInImages: boolean;
  builtInSubtitles: boolean;
  builtInArchives: boolean;
}

export const NO_OPTIONAL_ENGINES: EngineAvailability = {
  ffmpeg: false, libreOffice: false, qpdf: false, poppler: false, sevenZip: false, inkscape: false,
  builtInImages: true, builtInSubtitles: true, builtInArchives: true,
};

export interface OperationDefinition {
  id: string;
  title: string;
  description: string;
  category: OperationCategory;
  acceptedExtensions: readonly string[];
  outputLabel: string;
  status: OperationStatus;
  wheelMode?: WheelMode;
  targetFormat?: string;
  requiredEngine?: RequiredEngine;
  requiredEngines?: readonly RequiredEngine[];
  unavailableReason?: string;
}

const IMAGE_INPUTS = [".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tif", ".tiff"] as const;
const EXTRA_IMAGE_INPUTS = [".heic", ".heif", ".avif", ".gif"] as const;
const AUDIO_FORMATS = ["mp3", "m4a", "wav", "flac", "ogg", "opus", "aiff", "wma"] as const;
const VIDEO_FORMATS = ["mp4", "mov", "mkv", "webm", "avi", "wmv", "gif"] as const;
const AUDIO_EXTENSIONS = AUDIO_FORMATS.map((format) => "." + format);
const VIDEO_EXTENSIONS = VIDEO_FORMATS.filter((format) => format !== "gif").map((format) => "." + format);
const OFFICE_DOCUMENTS = [".docx", ".odt", ".rtf", ".txt", ".md", ".markdown", ".epub", ".html"];
const PRESENTATIONS = [".ppt", ".pptx", ".odp"];
const SPREADSHEETS = [".xlsx", ".xls", ".ods", ".csv", ".tsv"];

const builtinImageTargets = [
  { format: "png", title: "PNG" }, { format: "jpg", title: "JPG" },
  { format: "webp", title: "WebP" }, { format: "bmp", title: "BMP" },
  { format: "tiff", title: "TIFF" },
] as const;
const ffmpegImageTargets = [
  { format: "heic", title: "HEIC" }, { format: "avif", title: "AVIF" }, { format: "gif", title: "GIF" },
] as const;
const ffmpegImageOps: OperationDefinition[] = [
  ...builtinImageTargets.map(({ format, title }) => ({
    id: "ffmpeg-image-convert-" + format, title, description: "Convert HEIC/HEIF/AVIF/GIF input through FFmpeg.",
    category: "convert" as const, acceptedExtensions: EXTRA_IMAGE_INPUTS, outputLabel: title + " image",
    status: "available" as const, wheelMode: "convert" as const, targetFormat: format, requiredEngine: "ffmpeg" as const,
  })),
  ...ffmpegImageTargets.map(({ format, title }) => ({
    id: "ffmpeg-image-convert-" + format, title, description: "Export to HEIC/AVIF/GIF through FFmpeg; encoder availability depends on the installed build.",
    category: "convert" as const, acceptedExtensions: [...IMAGE_INPUTS, ...EXTRA_IMAGE_INPUTS], outputLabel: title + " image",
    status: "available" as const, wheelMode: "convert" as const, targetFormat: format, requiredEngine: "ffmpeg" as const,
  })),
];
const svgConversionOps: OperationDefinition[] = [
  { id: "svg-to-png", title: "PNG", description: "Render SVG to PNG through Inkscape.",
    category: "convert", acceptedExtensions: [".svg"], outputLabel: "PNG image",
    status: "available", wheelMode: "convert", targetFormat: "png", requiredEngine: "inkscape" },
  { id: "svg-to-pdf", title: "PDF", description: "Export SVG to PDF through Inkscape.",
    category: "convert", acceptedExtensions: [".svg"], outputLabel: "PDF document",
    status: "available", wheelMode: "convert", targetFormat: "pdf", requiredEngine: "inkscape" },
];
const audioConversionOps: OperationDefinition[] = AUDIO_FORMATS.map((format) => ({
  id: "media-audio-" + format, title: format.toUpperCase(), description: "Convert audio through FFmpeg.",
  category: "convert", acceptedExtensions: AUDIO_EXTENSIONS, outputLabel: format.toUpperCase() + " audio",
  status: "available", wheelMode: "convert", targetFormat: format, requiredEngine: "ffmpeg",
}));
const videoConversionOps: OperationDefinition[] = [
  ...VIDEO_FORMATS.map((format) => ({
    id: "media-video-" + format, title: format.toUpperCase(), description: "Convert video through FFmpeg.",
    category: "convert" as const, acceptedExtensions: VIDEO_EXTENSIONS, outputLabel: format.toUpperCase() + " video",
    status: "available" as const, wheelMode: "convert" as const, targetFormat: format, requiredEngine: "ffmpeg" as const,
  })),
  { id: "extract-audio-mp3", title: "MP3 export", description: "Extract MP3 audio from a video through FFmpeg.",
    category: "convert", acceptedExtensions: VIDEO_EXTENSIONS, outputLabel: "MP3 audio",
    status: "available", wheelMode: "convert", targetFormat: "mp3", requiredEngine: "ffmpeg" },
];
const COMPRESSIBLE_AUDIO_FORMATS = ["mp3", "m4a", "flac", "ogg", "opus", "wma"] as const;
const mediaCompressionOps: OperationDefinition[] = [...COMPRESSIBLE_AUDIO_FORMATS, ...VIDEO_FORMATS.filter((format) => format !== "gif")].map((format) => ({
  id: "media-compress-" + format, title: "Compress " + format.toUpperCase(),
  description: "Re-encode media locally; quality/size trade-offs apply.",
  category: "compress", acceptedExtensions: ["." + format], outputLabel: format.toUpperCase() + " media",
  status: "available", wheelMode: "tools", targetFormat: format, requiredEngine: "ffmpeg",
}));
const documentTargets = [
  { format: "pdf", title: "PDF" }, { format: "docx", title: "DOCX" }, { format: "odt", title: "ODT" },
  { format: "rtf", title: "RTF" }, { format: "txt", title: "TXT" }, { format: "html", title: "HTML" },
  { format: "epub", title: "EPUB" },
] as const;
const documentConversionOps: OperationDefinition[] = documentTargets.map(({ format, title }) => ({
  id: "office-convert-" + format, title, description: "Convert supported documents through an installed LibreOffice engine.",
  category: "convert",
  acceptedExtensions: format === "html" ? OFFICE_DOCUMENTS.filter((ext) => ![".txt", ".md", ".markdown"].includes(ext)) : OFFICE_DOCUMENTS,
  outputLabel: title + " document",
  status: "available", wheelMode: "convert", targetFormat: format, requiredEngine: "libreOffice",
}));
const textToHtmlOp: OperationDefinition = {
  id: "text-to-html", title: "HTML", description: "Render text or basic Markdown to a safe local HTML document.",
  category: "convert", acceptedExtensions: [".txt", ".md", ".markdown"], outputLabel: "HTML document",
  status: "available", wheelMode: "convert", targetFormat: "html",
};
const presentationConversionOps: OperationDefinition[] = [
  { id: "presentation-convert-pdf", title: "PDF", description: "Export a presentation to PDF through LibreOffice.",
    category: "convert", acceptedExtensions: PRESENTATIONS, outputLabel: "PDF slides",
    status: "available", wheelMode: "convert", targetFormat: "pdf", requiredEngine: "libreOffice" },
  { id: "presentation-convert-jpg", title: "JPG slides", description: "Render each presentation slide as a JPG through LibreOffice and Poppler.",
    category: "convert", acceptedExtensions: PRESENTATIONS, outputLabel: "JPG slide images",
    status: "available", wheelMode: "convert", targetFormat: "jpg", requiredEngines: ["libreOffice", "poppler"] },
  { id: "presentation-convert-png", title: "PNG slides", description: "Render each presentation slide as a PNG through LibreOffice and Poppler.",
    category: "convert", acceptedExtensions: PRESENTATIONS, outputLabel: "PNG slide images",
    status: "available", wheelMode: "convert", targetFormat: "png", requiredEngines: ["libreOffice", "poppler"] },
];
const spreadsheetOps: OperationDefinition[] = ["pdf", "xlsx", "ods", "csv", "tsv"].map((format) => ({
  id: "spreadsheet-convert-" + format, title: format.toUpperCase(), description: "Convert spreadsheet data through LibreOffice.",
  category: "convert", acceptedExtensions: SPREADSHEETS, outputLabel: format.toUpperCase() + " file",
  status: "available", wheelMode: "convert", targetFormat: format, requiredEngine: "libreOffice",
}));
const pdfOfficeOps: OperationDefinition[] = ["docx", "pptx", "xlsx", "epub"].map((format) => ({
  id: "pdf-office-" + format, title: format.toUpperCase(), description: "Attempt best-effort PDF import/export through LibreOffice; layout may differ.",
  category: "convert", acceptedExtensions: [".pdf"], outputLabel: format.toUpperCase() + " file",
  status: "available", wheelMode: "convert", targetFormat: format, requiredEngine: "libreOffice",
}));
const pdfImageOps: OperationDefinition[] = ["jpg", "png"].map((format) => ({
  id: "pdf-to-image-" + format, title: format.toUpperCase() + " pages", description: "Render every PDF page at 300 DPI.",
  category: "convert", acceptedExtensions: [".pdf"], outputLabel: format.toUpperCase() + " page images",
  status: "available", wheelMode: "convert", targetFormat: format, requiredEngine: "poppler",
}));
const imageToPdfOp: OperationDefinition = {
  id: "image-to-pdf", title: "PDF", description: "Convert a supported raster image to PDF using LibreOffice.",
  category: "convert", acceptedExtensions: [".png", ".jpg", ".jpeg", ".bmp", ".tif", ".tiff"],
  outputLabel: "PDF document", status: "available", wheelMode: "convert", targetFormat: "pdf", requiredEngine: "libreOffice",
};

export const OPERATIONS: readonly OperationDefinition[] = [
  { id: "create-zip", title: "Create ZIP", description: "Bundle files and folders into a validated DEFLATE archive.", category: "compress", acceptedExtensions: ["*"], outputLabel: "ZIP archive", status: "available", wheelMode: "tools" },
  { id: "create-tar", title: "Create TAR", description: "Bundle files and folders into a validated TAR archive.", category: "compress", acceptedExtensions: ["*"], outputLabel: "TAR archive", status: "available", wheelMode: "tools" },
  { id: "create-targz", title: "Create TAR.GZ", description: "Bundle and compress files/folders into a validated TAR.GZ archive.", category: "compress", acceptedExtensions: ["*"], outputLabel: "TAR.GZ archive", status: "available", wheelMode: "tools" },
  { id: "create-gzip", title: "Compress to GZIP", description: "Compress one file into a validated .gz stream.", category: "compress", acceptedExtensions: ["*"], outputLabel: "GZIP file", status: "available", wheelMode: "tools" },
  { id: "compress-jpeg", title: "Compress JPG", description: "Re-encode JPEG images locally at quality 78; preserve the original.", category: "compress", acceptedExtensions: [".jpg", ".jpeg"], outputLabel: "JPEG image", status: "available", wheelMode: "tools" },
  { id: "copy-paths", title: "Copy paths", description: "Copy selected local file paths as newline-separated text.", category: "action", acceptedExtensions: ["*"], outputLabel: "Clipboard text", status: "available", wheelMode: "actions" },
  { id: "open-folder", title: "Output folder", description: "Open the SHIFT output destination.", category: "action", acceptedExtensions: ["*"], outputLabel: "Folder", status: "available", wheelMode: "actions" },

  ...builtinImageTargets.map(({ format, title }) => ({
    id: "convert-" + format, title, description: "Convert supported raster images.",
    category: "convert" as const, acceptedExtensions: IMAGE_INPUTS, outputLabel: title + " image",
    status: "available" as const, wheelMode: "convert" as const, targetFormat: format,
  })),
  ...ffmpegImageOps,
  ...svgConversionOps,
  ...audioConversionOps,
  ...videoConversionOps,
  ...mediaCompressionOps,
  ...documentConversionOps,
  ...presentationConversionOps,
  ...spreadsheetOps,
  ...pdfOfficeOps,
  ...pdfImageOps,
  imageToPdfOp,
  textToHtmlOp,

  { id: "pdf-optimize", title: "Optimize PDF", description: "Recompress PDF streams and structure using qpdf.", category: "compress", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "available", wheelMode: "tools", requiredEngine: "qpdf" },
  { id: "pdf-merge", title: "Merge PDFs", description: "Combine two or more PDF files in selection order.", category: "organize", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "available", wheelMode: "tools", requiredEngine: "qpdf" },
  { id: "pdf-to-text-txt", title: "Extract TXT", description: "Extract searchable text from a PDF.", category: "convert", acceptedExtensions: [".pdf"], outputLabel: "TXT", status: "available", wheelMode: "convert", targetFormat: "txt", requiredEngine: "poppler" },
  { id: "pdf-to-text-md", title: "Extract Markdown", description: "Extract PDF text into a Markdown text file.", category: "convert", acceptedExtensions: [".pdf"], outputLabel: "Markdown", status: "available", wheelMode: "convert", targetFormat: "md", requiredEngine: "poppler" },
  { id: "subtitle-srt-vtt", title: "VTT", description: "Convert SRT subtitle timing to WebVTT.", category: "convert", acceptedExtensions: [".srt"], outputLabel: "VTT subtitles", status: "available", wheelMode: "convert", targetFormat: "vtt" },
  { id: "subtitle-vtt-srt", title: "SRT", description: "Convert WebVTT subtitle timing to SRT.", category: "convert", acceptedExtensions: [".vtt"], outputLabel: "SRT subtitles", status: "available", wheelMode: "convert", targetFormat: "srt" },
  { id: "subtitle-txt", title: "TXT captions", description: "Export subtitle text without timestamps.", category: "convert", acceptedExtensions: [".srt", ".vtt"], outputLabel: "TXT captions", status: "available", wheelMode: "convert", targetFormat: "txt" },
  { id: "archive-to-zip", title: "RAR / 7Z → ZIP", description: "Safely extract RAR/7Z contents and repackage them as ZIP.", category: "convert", acceptedExtensions: [".rar", ".7z"], outputLabel: "ZIP archive", status: "available", wheelMode: "convert", targetFormat: "zip", requiredEngine: "sevenZip" },
  { id: "tar-to-zip", title: "TAR / GZIP → ZIP", description: "Safely convert TAR, TAR.GZ, TGZ or single-file GZIP streams into ZIP.", category: "convert", acceptedExtensions: [".tar", ".tgz", ".gz"], outputLabel: "ZIP archive", status: "available", wheelMode: "convert", targetFormat: "zip" },
  { id: "planned-pdf-split", title: "Split PDF", description: "Split PDFs into page ranges.", category: "organize", acceptedExtensions: [".pdf"], outputLabel: "PDF", status: "planned", unavailableReason: "Page-range UI and split-output validation are still needed." },
];

export function extensionOf(filename: string): string {
  const leaf = filename.split(/[\\/]/).pop() ?? filename;
  const dot = leaf.lastIndexOf(".");
  return dot <= 0 ? "" : leaf.slice(dot).toLowerCase();
}

function targetAlreadyPresent(target: string | undefined, extensions: readonly string[]): boolean {
  if (!target) return false;
  if (target === "jpg") return extensions.some((ext) => ext === ".jpg" || ext === ".jpeg");
  if (target === "tiff") return extensions.some((ext) => ext === ".tif" || ext === ".tiff");
  return extensions.includes("." + target);
}

function supportsEngine(operation: OperationDefinition, engines: EngineAvailability): boolean {
  if (operation.requiredEngine && !engines[operation.requiredEngine]) return false;
  return !operation.requiredEngines || operation.requiredEngines.every((engine) => engines[engine]);
}

export function getAvailableOperations(fileNames: readonly string[], engines: EngineAvailability = NO_OPTIONAL_ENGINES): OperationDefinition[] {
  if (fileNames.length === 0) return [];
  const extensions = fileNames.map(extensionOf);
  return OPERATIONS.filter((operation) =>
    operation.status === "available"
    && supportsEngine(operation, engines)
    && (operation.acceptedExtensions.includes("*") || extensions.every((extension) => operation.acceptedExtensions.includes(extension)))
    && !(operation.category === "convert" && targetAlreadyPresent(operation.targetFormat, extensions))
    && !(operation.id === "pdf-merge" && fileNames.length < 2)
    && !(operation.id === "create-gzip" && fileNames.length !== 1),
  );
}

export function getWheelOperations(fileNames: readonly string[], mode: WheelMode, engines: EngineAvailability = NO_OPTIONAL_ENGINES): OperationDefinition[] {
  return getAvailableOperations(fileNames, engines).filter((operation) => operation.wheelMode === mode);
}

export function getOperationById(id: string): OperationDefinition | undefined {
  return OPERATIONS.find((operation) => operation.id === id);
}
