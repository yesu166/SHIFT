# Engine matrix

The wheel is driven by `src/lib/operations.ts`; the backend dispatcher is `src-tauri/src/engine_service.rs`. Menu visibility is gated by selected extensions and the required locally detected engine(s). A detectable program does not guarantee it was compiled with every encoder/codec; operation errors must remain visible.

| Family | Operation | Engine | Status / constraint |
|---|---|---|---|
| Images | PNG/JPG/WebP/BMP/TIFF conversion | Rust `image` crate | Built in; supported raster codecs only |
| Images | HEIC/AVIF/GIF conversion and selected output formats | FFmpeg | Optional; encoder/decoder support varies by build |
| Images | SVG to PNG/PDF | Inkscape | Optional |
| Images/PDF | Raster image to PDF | LibreOffice | Optional; raster source formats registered in the wheel |
| Image compression | JPEG quality-78 re-encode | Rust `image` crate | Built in, lossy; may grow some files |
| Audio/video | Convert MP3/M4A/WAV/FLAC/OGG/Opus/AIFF/WMA, MP4/MOV/MKV/WebM/AVI/WMV and GIF; video-to-MP3 | FFmpeg | Optional, codecs vary by build |
| Audio/video | Size-focused re-encode of selected audio/video formats | FFmpeg | Optional; quality can decrease and size is not guaranteed to shrink |
| PDF | Object-stream and Flate recompression; PDF merge | qpdf | Optional; this is structural optimization, not image OCR |
| PDF | Pages to JPG/PNG at 300 DPI; extract TXT/Markdown | Poppler | Optional; image-only PDFs require OCR, not included |
| Documents | Office-compatible document conversion | LibreOffice | Optional; best effort, layout may change |
| Slides | PPT/PPTX/ODP to PDF | LibreOffice | Optional |
| Slides | Each slide rendered to JPG/PNG | LibreOffice + Poppler | Both required |
| Spreadsheets | XLS/XLSX/ODS/CSV/TSV conversions | LibreOffice | Optional; CSV is single-sheet data, layout may not survive |
| Subtitles | SRT ↔ VTT; SRT/VTT to TXT | Built-in transformer | UTF-8 only; no OCR, speech recognition or translation |
| HTML | TXT/Markdown to escaped HTML | Built-in transformer | Basic headings/lists/fenced code; does not execute source HTML |
| Archives | ZIP | Rust `zip` crate | Built in, DEFLATE level 6, read-back validation |
| Archives | TAR/TAR.GZ/GZIP creation; TAR/GZIP to ZIP | Rust `tar` + `flate2` | Built in; validated and bounded |
| Archives | RAR/7Z to ZIP | 7-Zip | Optional; path/link/size/count/depth guards then validated ZIP writer |

## Safety boundaries

- External commands receive argument arrays, not an interpolated shell string.
- Windows child processes run without creating a console window.
- Tool output excerpts are bounded. Staged outputs must exist and be non-empty; PDF, image and Office package outputs receive format-appropriate structural checks.
- ZIP output is reopened and every regular entry is read.
- Archive conversion rejects absolute paths, parent-relative paths, link entries, paths deeper than 64 components, paths longer than 32,000 bytes, more than 100,000 entries, and over 2 GiB of declared expanded content. TAR/GZIP extraction rejects special/link entries, duplicate paths, non-normal path components and the same count/size/path bounds.
- Source files are preserved; the app writes new output names and will not intentionally clobber existing outputs.
- PDF/Office conversions can change layout. A readable header or container is structural validation, not a proof of visual fidelity.

## Not connected yet

PDF split-by-page-range, watermark, password protection/removal, redaction, OCR, audio transcription, faithful Apple Pages conversion, arbitrary-folder archive extraction and a complete animation/metadata-preserving image pipeline are not implemented. Unsupported formats must not be advertised as guaranteed round-trips.
