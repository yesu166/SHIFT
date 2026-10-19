# Implementation status

Last reviewed: 2026-10-19

## Architecture

- React/TypeScript/Vite renders the workspace and separate transparent always-on-top wheel.
- Tauri 2/Rust owns OS integration and processing.
- The typed registry (`src/lib/operations.ts`) describes formats, gesture mode and optional required engines.
- `src-tauri/src/engine_service.rs` detects optional executables and dispatches jobs on Tauri's blocking pool. No separate HTTP server is needed.
- Core processing is local. External engines are not downloaded by SHIFT.

## Built-in capabilities

- Raster image conversion among PNG/JPG/WebP/BMP/TIFF, output decoding validation and collision-safe output names.
- JPEG re-encode at quality 78 (lossy; output size can grow).
- ZIP creation/validation with Rust `zip`, plus TAR/TAR.GZ/single-file GZIP and TAR/GZIP-to-ZIP.
- SRT↔VTT and subtitle text extraction.
- Safe basic TXT/Markdown-to-HTML export with HTML escaping.
- New outputs preserve the original input.

## Optional engine integrations

| Engine | Registered capability | Detection |
|---|---|---|
| FFmpeg | Audio/video conversion, selected media compression, advanced image conversion, video audio export | `ffmpeg -version` |
| LibreOffice | Office/document/spreadsheet/slides conversions; raster images to PDF | `soffice --version` or `libreoffice --version` |
| qpdf | PDF structural optimization and merge | `qpdf --version` |
| Poppler | PDF-to-page images and text extraction; required with LibreOffice for slide-image export | `pdftoppm -v` and `pdftotext -v` |
| 7-Zip | RAR/7Z to ZIP | `7z -h`, `7zz -h` or `7za -h` |
| Inkscape | SVG to PNG/PDF | `inkscape --version` |

Codec availability, import filters and actual output fidelity can vary even when the executable is detected. The engine reports failures rather than claiming a result.

## Verification gates

- Frontend typecheck, lint, unit tests and production build.
- Windows Rust format, unit tests, Clippy and NSIS installer build in CI.
- Manual Windows tests remain necessary for Explorer OLE drag continuity, overlay focus behavior, modifier switching, result popovers, tray lifecycle, high-DPI and multiple displays.
- Verify optional-engine conversions with representative samples on Windows, including absent engines, bad inputs, codec errors, filenames with Unicode/spaces, large files, locked output paths and disk-full conditions.
- Visual/fidelity validation is required for PDF and Office conversions; parsing a header/container alone is not enough.

## Not connected

PDF split ranges, watermark/password protection/removal, redaction, OCR, audio transcription, arbitrary-folder extraction UI, faithful Apple Pages conversion and guaranteed animation/metadata preservation are not claimed.
