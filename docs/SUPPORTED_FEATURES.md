# Supported features

The typed registry in src/lib/operations.ts is the single source of truth. Only ZIP creation is currently marked available.

| Feature | Inputs | Output | Status | Verification gate |
|---|---|---|---|---|
| Global activation key | F6–F12; F8 default | Shows the overlay near the pointer | IMPLEMENTED IN SOURCE | Verify registration and key conflict handling on Windows. |
| Always-on-top wheel | Current desktop | Separate transparent native window | IMPLEMENTED IN SOURCE | Verify focus preservation and DPI/multi-monitor behavior. |
| Drag a file to the ZIP action | Files/directories from Explorer | Validated DEFLATE ZIP | IMPLEMENTED IN SOURCE; MANUAL TEST REQUIRED | Verify native drop and OLE drag continuity on Windows. |
| Tray-resident behavior | Close main app window | Hide main window; tray can reopen or quit | IMPLEMENTED IN SOURCE | Validate tray lifecycle and exit behavior. |
| Compress files to ZIP | Files and directories | DEFLATE ZIP | IMPLEMENTED | CI engine tests and independent Windows extraction must pass before release. |
| Optimize PDF | PDF | Optimized PDF | PLANNED | Real optimization engine, output parsing, actual byte comparison and source preservation. |
| Merge PDFs | PDFs in selected order | PDF | PLANNED | Reopen output and verify page count/order. |
| Split or extract pages | PDF and page selection | PDF file(s) | PLANNED | Validate ranges and output page structure. |
| Rotate, reorder, remove pages | PDF and page options | PDF | PLANNED | Verify output page structure/orientation. |
| DOCX/XLSX/PPTX to PDF | Office document | PDF | PLANNED | Optional engine detection and representative layout/fidelity tests. |
| Page numbers/watermarks | PDF and settings | PDF | PLANNED | Real PDF writer and fidelity tests. |
| PDF password protection/removal | PDF and the required password | Encrypted/decrypted PDF | PLANNED | Secure password flow plus successful output validation. |
| ZIP extraction | ZIP | Files/folders | NOT_SUPPORTED | Implement path traversal, symlink, expansion size, count/depth and overwrite protections first. |
| Cross-application drag-to-wheel | OS-originating drag | Action wheel | IMPLEMENTED IN SOURCE; MANUAL TEST REQUIRED | Validate Explorer drag continuity, focus preservation, and target hit-testing. |
| Start at Windows sign-in | User session | Tray-resident app | NOT IMPLEMENTED | Add explicit opt-in sign-in startup support later. |

The browser preview cannot pass a browser File as a native path. ZIP processing requires the Tauri desktop shell and native paths.
