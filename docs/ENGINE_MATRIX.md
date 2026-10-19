# Engine matrix

Engine selection depends on actual capability, version, license, notices, security behavior, output validation and redistributed footprint. Only items marked implemented below are connected to the wheel.

| Purpose | Engine | Input / output | Status |
|---|---|---|---|
| Image conversion | Rust image crate with JPEG, PNG, WebP, BMP and TIFF features | PNG/JPG/JPEG/WebP/BMP/TIFF to a different supported image format | IMPLEMENTED IN SOURCE; CI REQUIRED |
| JPEG compression | Rust image JPEG encoder | JPG/JPEG to a new JPG at quality 78 | IMPLEMENTED IN SOURCE; CI REQUIRED |
| ZIP archive creation and read-back validation | Rust zip crate, pinned 8.6.0 | Files/folders to DEFLATE ZIP; read back every regular entry | IMPLEMENTED IN SOURCE |
| Directory traversal | walkdir | Folder recursion with symlink traversal disabled | IMPLEMENTED IN SOURCE |
| Temporary outputs | tempfile | Staging before finalization | IMPLEMENTED IN SOURCE |
| Copy file paths | WebView clipboard API | Selected native paths to text | IMPLEMENTED IN SOURCE; MANUAL TEST REQUIRED |
| Open output location | Rust open crate | Open a generated output's parent folder | IMPLEMENTED IN SOURCE |
| Audio/video conversion and compression | Optional FFmpeg integration | Common media formats | PLANNED; NOT CONNECTED |
| PDF render/create/edit | PDF engine to be selected | Image-to-PDF, merge, split, optimize, watermark, protect | PLANNED; NOT CONNECTED |
| Office conversion | Optional LibreOffice or another reviewed engine | DOCX/XLSX/PPTX conversion | PLANNED; NOT CONNECTED |

## Image engine details

- Source formats accepted by built-in image operations: PNG, JPG/JPEG, WebP, BMP and TIFF.
- Destination formats: PNG, JPG/JPEG, WebP, BMP and TIFF.
- The wheel removes a target choice if any source in the current selection is already in that target format, avoiding silent no-op batch items.
- Each conversion is written to a temporary sibling file, decoded again for validation, then saved under a new collision-safe name.
- JPEG compression is a lossy quality-78 re-encode. It may make a small image larger; report measured size rather than promise a reduction.
- Full metadata preservation, animation/GIF support, HEIC/AVIF/SVG input and ICC-profile fidelity are not promised by this engine.

## ZIP engine details

- DEFLATE compression at level 6; stream bytes rather than loading all source files into memory.
- Supports files, nested folders and empty directory entries.
- Rejects selected symbolic-link roots, skips symbolic links inside selected folders, and rejects overlapping selections or duplicate archive paths.
- Limits selection to 1,000 root paths and 100,000 entries.
- Reopens the temporary archive and reads each regular entry before finalization.
- Measures input/output size honestly; ZIP may be larger than the original data.
- Independent extraction, large-file stress and Windows Explorer drag-drop tests remain release gates.

## External dependency policy

Do not bundle large command-line engines simply to claim format coverage. Prefer feature detection, clear availability status, safe argument passing rather than shell-string concatenation, local output validation, explicit licensing/notice review and a measurable installer-size budget. If an external engine is absent, its menu items should be unavailable with a clear explanation.
