# Supported features

This page describes what is connected to the current SHIFT wheel—not a future wish list.

| Feature | Inputs | Output | Status / note |
|---|---|---|---|
| Contextual drag wheel | File Explorer drag with Shift/Alt+Shift/Ctrl+Shift | Convert, tools or file actions | IMPLEMENTED IN SOURCE; needs hands-on Windows validation |
| Convert images | PNG, JPG/JPEG, WebP, BMP, TIFF | PNG, JPG/JPEG, WebP, BMP, TIFF | BUILT-IN; target hidden if already present in the batch |
| Compress JPEG | JPG/JPEG | Re-encoded JPG/JPEG at quality 78 | BUILT-IN; lossy; output size may increase |
| Create ZIP | Files and folders | DEFLATE ZIP in Documents/SHIFT/Archives | BUILT-IN; archive is reopened and its regular entries read |
| Copy paths | Dropped selection | Newline-separated local paths on clipboard | BUILT-IN; clipboard behavior needs manual Windows test |
| Open output destination | Wheel action or result | Explorer opens the folder | BUILT-IN |
| Tray-resident behavior | Close main window | Main window hides to tray; tray can reopen or quit | IMPLEMENTED IN SOURCE; lifecycle needs manual verification |
| GIF animation | GIF input | Animated image conversion | NOT SUPPORTED by current built-in engine |
| HEIC/HEIF/AVIF/SVG conversion | These source formats | Images | NOT SUPPORTED by current built-in engine |
| Image metadata fidelity | Images with EXIF/ICC metadata | Same metadata copied verbatim | NOT GUARANTEED; output must be checked independently |
| Audio/video conversion/compression | Media formats | Converted/compressed media | PLANNED; FFmpeg integration not connected |
| PDF create/merge/split/optimize | PDF or supported source files | PDF files | PLANNED; no PDF engine connected |
| DOCX/XLSX/PPTX conversion | Office files | PDF or other formats | PLANNED; optional engine not connected |
| Password protection, watermarking, redaction | Documents and user options | Modified documents | PLANNED |
| Start at Windows sign-in | Windows user session | Tray-resident app | NOT IMPLEMENTED |

The browser preview cannot pass browser File objects as native filesystem paths. Native path operations require the Tauri desktop shell. Unsupported actions must stay out of the executable menu.
