# SHIFT

SHIFT is a Windows-first, local-first file wheel that brings compatible file operations to the pointer while a file is being dragged. It uses Rust, Tauri 2, React and TypeScript.

The product direction is inspired by contextual file wheels, but SHIFT has its own implementation, copy, visual identity and feature roadmap. It is not affiliated with Tangerine.

## The workflow

Start dragging a file from File Explorer, then tap a modifier while the mouse button is still held:

- **Shift — Convert.** Convert images among PNG, JPEG, WebP, BMP and TIFF. The wheel hides the format the selection is already in.
- **Alt + Shift — Tools.** Re-encode JPEG images at quality 78, or bundle files and folders into ZIP.
- **Ctrl + Shift — File actions.** Copy selected file paths to the clipboard or open SHIFT's output destination.
- **Escape — Cancel.** The wheel is designed not to take focus away from Explorer.

The menu is contextual: it shows only operations compatible with the current selection. A conversion writes a new sibling file next to the source. ZIP archives are written to Documents/SHIFT/Archives. Existing files are not intentionally replaced, and processing errors are surfaced instead of reporting fake success.

## Built-in engines

| Engine | Supported work | Implementation |
|---|---|---|
| Image conversion | PNG, JPG/JPEG, WebP, BMP, TIFF | Rust image crate with output decoding/validation |
| JPEG compression | JPEG files only; lossy quality setting currently 78 | Rust JPEG encoder; keeps the original untouched |
| ZIP archive | Files, folders, nested files and empty folders | Rust zip crate, DEFLATE level 6, temporary output and read-back validation |
| File actions | Copy paths; open destination folders | Clipboard API and native Windows file handling |

**Not implemented yet:** AVIF/HEIC/GIF/SVG support in the built-in image engine, PDF creation or optimization, PDF structural editing, Office conversion, and audio/video conversion. These will remain unavailable until there is a real engine, a safe output flow and representative output tests. Larger optional dependencies such as FFmpeg or LibreOffice may be considered later; they are not silently assumed to be installed.

## SHIFT's different angle

- **Native and lean by design:** Rust processing with a Tauri UI, not a cloud wrapper.
- **Private by default:** core file processing is local; there is no account or upload dependency.
- **Compatible actions only:** the wheel does not offer a target format identical to the current input.
- **Originals are preserved:** each conversion produces a separate output and ZIP is validated before success is shown.
- **Clear feature boundaries:** unsupported formats are not represented as working buttons.

## Current verification status

Automated typecheck, lint, unit tests, Rust tests/Clippy and the Windows installer build run in SHIFT CI. **A real Explorer drag-and-drop test is still required on a Windows desktop**—in particular, checking that modifiers are detected, the transparent window does not disrupt the drag source, the selected petal receives the drop, and the wheel does not disappear before the job starts. A successful compile is not proof that the native interaction works on every Windows/DPI configuration.

## Stack

- React, TypeScript, Vite
- Tauri 2, Rust
- image crate for local image decoding/encoding
- zip plus DEFLATE, walkdir and tempfile for safe archives
- Tauri native file picker, tray, webview events and global shortcut support

## Development

Install Node.js/npm and Rust stable. On Windows, follow the Tauri 2 prerequisites at https://v2.tauri.app/start/prerequisites/, including Microsoft C++ Build Tools, Windows SDK and WebView2.

    npm install
    npm run typecheck
    npm run lint
    npm test
    npm run build
    npm run tauri:dev

Build the NSIS installer on Windows with npm run tauri:build. The browser dev server is preview-only; native paths and file processing require the Tauri desktop app.

## Safety and licensing

- Source images are read, not modified; converted outputs use new names next to the source.
- Archive names are collision-safe; selected symlink roots are rejected and symlinks encountered inside folders are skipped.
- ZIP selection and entry counts are bounded, and the completed archive is reopened and read before success.
- Core operations require no account, cloud service or backend.
- SHIFT's own redistribution license and final transitive dependency/license inventory must be resolved before public release. See THIRD_PARTY_NOTICES.md and the security documentation.
