# Architecture

## File operation flow

The main workspace and native wheel share the central typed operation registry in `src/lib/operations.ts`. The registry filters operations by the selected file extensions, gesture mode and the detected availability of required local engines.

The native command dispatcher (`src-tauri/src/lib.rs`) routes built-in raster image operations to `conversion_service.rs`, ZIP creation to `zip_service.rs`, and wider conversion/tool operations to `engine_service.rs`. Subprocess work and large file operations run on Tauri's blocking worker pool.

## Engine choices

- Built-in, size-conscious Rust code handles core raster image conversion, ZIP, TAR/TAR.GZ/GZIP, SRT/VTT and safe basic text-to-HTML output.
- Optional local executables extend format breadth: FFmpeg for media/advanced codecs, LibreOffice for office-family conversion, qpdf/Poppler for PDF, 7-Zip for RAR/7Z, and Inkscape for SVG.
- SHIFT does not silently install or bundle optional engines. Settings show detection status; an operation's failure is surfaced with a bounded diagnostic.
- The wheel offers only combinations registered for the selected format/gesture and engine availability. FFmpeg encoder/codec presence and LibreOffice import/export fidelity can still vary by installation.

## Safety

- Calls pass separate command arguments and do not interpolate shell strings.
- Windows child processes are configured not to create console windows.
- Generated outputs are staged and checked for basic structural validity before being moved to collision-safe names. ZIPs are reopened and each regular entry is read.
- Archive extraction rejects absolute/parent paths, links and special entries; limits entries, total expansion, path size and path depth.
- Original inputs are never intentionally overwritten. Local operations do not require an account or upload.

## Scope boundaries

The service is not a sandbox or antivirus. Structural checks do not establish malware safety or visual fidelity. PDF split, watermark/password operations, OCR, speech-to-text, arbitrary extraction-to-folder UI and faithful Apple Pages conversion are not implemented. Explorer drag/drop, focus preservation, tray lifecycle and DPI behavior still require manual Windows verification.
