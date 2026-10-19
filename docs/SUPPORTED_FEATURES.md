# Supported features

The table shows operations connected to the current branch. Optional-engine items appear in the wheel only when the required tool(s) are detected in PATH.

| Family | Inputs | Outputs/actions | Engine |
|---|---|---|---|
| Raster images | PNG, JPG/JPEG, WebP, BMP, TIFF | Convert among PNG, JPG, WebP, BMP and TIFF | Built-in Rust image engine |
| Advanced images | HEIC, HEIF, AVIF, GIF and supported raster inputs | Convert to/from HEIC/AVIF/GIF where encoder exists | Optional FFmpeg |
| Vector graphics | SVG | PNG or PDF | Optional Inkscape |
| Images to PDF | PNG, JPG/JPEG, BMP, TIFF | PDF | Optional LibreOffice |
| JPEG compression | JPG/JPEG | Quality-78 re-encode to a new file | Built in, lossy |
| Audio | MP3, M4A, WAV, FLAC, OGG, Opus, AIFF, WMA | Conversion to supported registered audio targets | Optional FFmpeg; encoders vary |
| Video | MP4, MOV, MKV, WebM, AVI, WMV | Convert among registered video targets or export audio to MP3 | Optional FFmpeg |
| Media compression | Selected MP3/M4A/FLAC/OGG/Opus/WMA and video formats | Size-focused re-encode | Optional FFmpeg; quality trade-offs apply |
| PDF tools | PDF | Optimize streams; merge two or more PDFs | Optional qpdf |
| PDF export/extraction | PDF | Every page to JPG/PNG at 300 DPI; extract text to TXT/Markdown | Optional Poppler |
| Documents | DOCX, ODT, RTF, TXT, Markdown, HTML, EPUB and other registered office-compatible inputs | PDF/DOCX/ODT/RTF/TXT/HTML/EPUB and selected registered office outputs | Optional LibreOffice; best effort |
| Slides | PPT, PPTX, ODP | PDF; each slide to JPG/PNG | LibreOffice; Poppler also required for images |
| Spreadsheets | XLS, XLSX, ODS, CSV, TSV | PDF/XLSX/ODS/CSV/TSV | Optional LibreOffice |
| Subtitles | SRT, VTT | SRT ↔ VTT and TXT caption export | Built in, UTF-8 |
| Text/web | TXT, MD, Markdown | Safe basic HTML export | Built in |
| Archives | Files/folders | ZIP, TAR, TAR.GZ; single-file GZIP | Built in |
| Archives | TAR, TGZ, TAR.GZ, GZIP | ZIP | Built in; bounded safe extraction |
| Archives | RAR, 7Z | ZIP | Optional 7-Zip; guarded extraction |

## Important limits

- The UI does not promise every source/target pairing. It offers registered actions for the current selection and detected engines.
- FFmpeg codec/encoder support varies by build. Detecting FFmpeg does not guarantee HEIC/AVIF/WMA or every requested codec; failures surface in the wheel.
- GIF to a still image exports one frame. Animated GIF creation from video is supported, but animation/metadata preservation across other formats is not promised.
- PDF-to-office output is best-effort and may substantially alter layout. Scanned/image-only PDF OCR is not built in.
- Apple Pages is not treated as a supported Windows input/output format. PDF split, watermarking, password tools, redaction and OCR are not connected yet.
- Optional engine executables are not bundled by default. See `README.md` for official download sources and setup.
- Native Explorer OLE drag/drop, modifier timing, focus behavior, DPI/multi-monitor placement and tray behavior still require hands-on Windows testing.
