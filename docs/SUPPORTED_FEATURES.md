# Supported features matrix

All user-facing operations remain unavailable until implemented and tested with real outputs. Planned ideas must not be presented as working capabilities.

| Feature | Inputs | Output | Status | Required evidence before enabling |
|---|---|---|---|---|
| Compress Files to ZIP | Files/folders | Standard compressed ZIP | PLANNED | DEFLATE, Unicode/nested paths, output integrity, Windows-compatible extraction, actual metrics. |
| PDF merge | PDF files | PDF | PLANNED | Reopen output; check page count/order; preserve sources. |
| PDF split/extract | PDF + page selection | PDF file(s) | PLANNED | Validate ranges, output page counts, and content order. |
| PDF rotate/reorder/remove | PDF + page operations | PDF | PLANNED | Reopen output and verify page count/orientation/order. |
| Optimize PDF | PDF | Optimized PDF | PLANNED | Structural validation and measured before/after bytes; output may grow. |
| DOCX to PDF | DOCX | PDF | PLANNED | Verified engine and representative rendering/fidelity tests. |
| XLSX to PDF | XLSX | PDF | PLANNED | Verified engine; test print areas, sheets, formulas, fonts and layout. |
| PPTX to PDF | PPTX | PDF | PLANNED | Verified engine; test slides, images, fonts and layouts. |
| PDF password protection | PDF + user-supplied password | Encrypted PDF | PLANNED | Supported engine and encryption/output reopening checks; never log password. |
| PDF password removal | Encrypted PDF + supplied password | PDF | PLANNED | Supported engine, correct password validation and source preservation. |
| Page numbers/watermarks | PDF + options | PDF | PLANNED | Actual PDF-writing implementation; renderer alone is insufficient. |
| ZIP extraction | ZIP | Files/folders | PLANNED | Zip Slip/symlink protections, bounds, collision handling, safe limits and independent tests. |
| Cross-app Shift-drag | OS interaction | Action menu | BLOCKED | Windows proof of concept that obtains the actual dragged payload safely and reliably. |

No feature is currently marked TESTED or VERIFIED_ON_WINDOWS.
