# Security and permissions

## Current design

- Core work is local; SHIFT has no required cloud API, login or upload dependency.
- The frontend can invoke named native commands for ZIP/image processing, engine detection and engine operations. It has no arbitrary shell command bridge.
- External commands are launched with discrete argument arrays, not shell interpolation. Windows subprocesses are launched without a visible console window.
- Engines run only when the user has independently installed them and placed their executable directory on PATH.
- External process output is truncated to bounded error excerpts.
- Image/PDF/Office outputs receive format-specific structural checks; ZIP output is reopened and every regular entry is read; TAR/GZIP output is parsed/decompressed for validation.
- Archive extraction checks for absolute/parent-relative paths, symbolic/hard links and special entries. It limits expanded content to 2 GiB, 100,000 entries, path depth to 64 components and path length to 32,000 bytes.
- Collision-safe outputs preserve source files; temporary outputs are finalized using non-clobbering persistence.
- The app's Tauri process remains tray-resident when the main window closes; there is no Windows Service Control Manager service or sign-in autostart.

## Optional tools

FFmpeg, LibreOffice, qpdf, Poppler, 7-Zip and Inkscape are separate third-party software programs. They are not silently installed by SHIFT, and may have their own license terms, updates, codecs or plugins. Use official/reputable distribution sources, keep them updated, and do not assume that executable detection implies all formats/encoders are available.

## Remaining review

- Complete direct/transitive dependency license inventory and choose SHIFT's own redistribution license before public release.
- Test malformed archive inputs, corrupted codecs, invalid PDFs, unsupported LO filters, Unicode/spaces, path traversal, links, large files, disk-full/locked files and concurrent operations.
- Manually test Explorer drag/drop, keyboard gesture timing, focus behavior, result actions, tray lifecycle and multi-monitor/high-DPI placement.
- Fidelity testing is required for Office/PDF jobs; basic file-format validation cannot prove a document looks the same as the source.
