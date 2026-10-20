# SHIFT

SHIFT is a Windows-first, local-first contextual file wheel. Start dragging a file from the Windows Desktop or File Explorer, hold a configurable modifier combination, and the animated wheel appears near the pointer. Move over a supported conversion or tool and release the drag to run it. SHIFT saves the result beside the original as a new file and never intentionally replaces the source. The native application is built with Rust, Tauri 2, React and TypeScript. The wheel can appear over different parts of the virtual desktop; its position is bounded to keep it on the active monitor.

SHIFT borrows the low-interruption *interaction pattern* of a contextual file wheel, not Tangerine's branding or assets. SHIFT is an independent project.

## Windows drag gestures

Shortcuts can be changed in **Settings → Drag shortcuts** and are stored locally. The wheel size can be adjusted from **Settings → Wheel size** (360–720 logical pixels); the selected size applies the next time the wheel opens.

- **Shift — Convert:** show compatible output formats for the dragged files.
- **Alt + Shift — Tools:** compression and archive/PDF tools.
- **Ctrl + Shift — File actions:** copy paths or open SHIFT's output destination.
- **Escape — Cancel.**

SHIFT resolves the drag source from the native Windows Desktop/File Explorer window under the pointer at drag start, rather than relying on the foreground window later in the drag. That allows the wheel to appear at the active pointer location even if Windows reports a different foreground HWND during the drag. The current global pointer monitor detects mouse drags before it knows their OLE payload, so the first end-to-end test should be a real file dragged from the Desktop or File Explorer. Modifier combinations can affect normal Windows drag semantics; adjust them in Settings if a default conflicts with your workflow.

The wheel uses the operation registry and local tool availability reported by the Rust engine service. It displays every matching action for the current selection, shrinking action tiles when needed to avoid crowding. If the current mode has no compatible action, it names the unsupported file type and shows the configured shortcuts for the other modes. Operations that require missing engines are not offered as executable actions.

## Format and engine matrix

“Built in” means the implementation is in SHIFT. “Optional local engine” means that program must be installed separately, available on PATH, and compatible with the requested codec/format.

| Family | Connected operations | Engine / limits |
|---|---|---|
| Images | PNG, JPG/JPEG, WebP, BMP and TIFF conversion; JPEG re-encoding; SVG to PNG/PDF; optional HEIC/AVIF/GIF conversion and HEIC/AVIF/GIF export | Rust `image` crate for raster codecs; Inkscape for SVG; FFmpeg for advanced formats/encoders. HEIC/AVIF encoders vary by FFmpeg build. GIF-to-still-image conversion uses a frame, not an animation-preserving round trip. |
| Audio | MP3, M4A, WAV, FLAC, OGG, Opus, AIFF and WMA conversion; selected formats have size-focused re-encoding | Optional FFmpeg. Not every FFmpeg build includes every encoder. WAV/AIFF are not advertised as size-reducing compression. |
| Video | MP4, MOV, MKV, WebM, AVI, WMV and GIF conversion; export MP3 audio; compression with format-specific encoder settings | Optional FFmpeg. Codec/profile/encoder support depends on the installed build. Compression may reduce quality and may not reduce every file. |
| PDF | Optimize PDF streams; merge selected PDFs; export every page to JPG/PNG at 300 DPI; extract searchable text to TXT/Markdown | qpdf for structural compression/merge, Poppler for page rendering/text extraction. Scanned image-only PDFs need OCR, which is not included. |
| Documents | Convert supported DOCX/ODT/RTF/TXT/HTML/EPUB and related office-compatible formats among the outputs registered in the wheel; TXT/Markdown to safe basic HTML | Optional LibreOffice for document conversions; built-in escaped-text Markdown/TXT-to-HTML converter. Layout and format support vary by input. Apple Pages is not promised on Windows. |
| Slides | PPT/PPTX/ODP to PDF; export each slide as JPG/PNG | LibreOffice plus Poppler are both required for slide images. |
| Spreadsheets | XLS/XLSX/ODS/CSV/TSV conversion to PDF/XLSX/ODS/CSV/TSV | Optional LibreOffice. CSV exports can represent only one sheet at a time; formula/layout fidelity depends on the source and target. |
| Subtitles | SRT ↔ VTT; SRT/VTT to plain TXT | Built-in UTF-8 text/timestamp transformer. It does not perform speech recognition or translate captions. |
| Archives | ZIP, TAR, TAR.GZ and single-file GZIP creation; TAR/TGZ/TAR.GZ/GZIP to ZIP; RAR/7Z to ZIP | ZIP is created and validated with the existing Rust service. TAR/GZIP are built in. RAR/7Z needs 7-Zip. Extraction rejects link entries, unsafe paths, more than 100,000 entries, paths deeper than 64 components, paths longer than 32,000 bytes, and declared content above 2 GiB. |

### Important boundaries

SHIFT does **not** claim every possible source/target pair. The wheel lists only combinations registered for that family and whose named engine is detected. Installed FFmpeg codec support can still vary, so a conversion may fail with a visible engine error rather than report a false success.

Not connected yet: PDF split/page extraction by range, PDF watermark/password protection/redaction, audio transcription/OCR, faithful conversion of proprietary Apple Pages files, archive extraction directly to arbitrary folders, and animation/metadata fidelity guarantees. PDF → office formats is best-effort and can significantly alter layout.

## Optional local engines

SHIFT does not silently download, install or bundle these large tools. Install only the engines needed for your workflow, add their executable directory to Windows `PATH`, then fully restart SHIFT:

- FFmpeg: https://ffmpeg.org/download.html
- LibreOffice: https://www.libreoffice.org/download/download-libreoffice/
- qpdf: https://qpdf.sourceforge.io/
- Poppler tools (`pdftoppm` and `pdftotext`): https://poppler.freedesktop.org/
- 7-Zip: https://www.7-zip.org/
- Inkscape: https://inkscape.org/release/

The **Settings → System status** section shows which tools SHIFT detects. Detection checks executable availability; encoder/filter availability can still differ by build. Never download engine executables from untrusted mirrors.

## Output, safety and privacy

- Core work happens locally and requires no account or upload service.
- Image and engine-based conversions save new sibling files using names such as `photo (SHIFT converted).png`; numbered suffixes avoid replacing existing outputs. ZIP output is saved to Documents/SHIFT/Archives.
- Generated outputs are staged, measured, and validated for their relevant file structure before finalization. Existing outputs are not intentionally overwritten.
- Media processes use argument arrays rather than shell-string interpolation, run without a console window on Windows, and keep bounded error excerpts.
- Archive conversion applies input bounds and rejects unsafe paths and link entries. A file on disk can still be malformed or malicious; validation is not a malware scanner.
- External engine programs are separate third-party applications with their own licenses and security-update schedules.

## Current verification status

Frontend checks, Rust tests/Clippy and Windows installer creation run in [SHIFT CI](.github/workflows/ci.yml). A successful build does not establish working Desktop/Explorer OLE drag/drop, click-through behavior around the overlay, focus preservation, multi-monitor placement, DPI scaling, or every external engine/codec. Those still need hands-on verification on a Windows desktop. The wheel entrance and radial action tiles use separate short animations; the Reduce motion setting disables those animations.

## Development

Install Node.js/npm and Rust stable. On Windows, follow Tauri 2 prerequisites: https://v2.tauri.app/start/prerequisites/

```sh
npm ci
npm run typecheck
npm run lint
npm test
npm run build
npm run tauri:dev
```

Build the NSIS installer on Windows with `npm run tauri:build`.

## Licensing

SHIFT's own redistribution license and complete direct/transitive third-party notices still need final review before redistribution. Optional programs are not bundled by default. See `THIRD_PARTY_NOTICES.md` and the security documentation.
