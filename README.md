# SHIFT

SHIFT is a Windows-first, local-first document toolbox built with React, TypeScript, Vite, Tauri 2 and Rust.

The first workflow under implementation is **Compress files to ZIP**: choose files or folders, stream them into a DEFLATE archive, reopen the archive and read all regular entries for validation, measure the actual input/output size, and save under Documents/SHIFT/Archives without replacing an existing output.

## Current status
- Implemented in source: React workspace, shared operation registry, quick-action wheel, light/dark theme, reduced-motion setting, native file picker/drop bridge, session history and Rust ZIP engine.
- CI runs automatically on pushes and pull requests. A source commit is not proof that tests passed; check [SHIFT CI](.github/workflows/ci.yml) and its workflow status.
- Not yet verified: packaged Windows app, installer, cross-monitor/DPI behavior or independent Windows Explorer extraction.
- PDF actions, password protection and Office conversion remain unavailable until real engines and output tests are added.
- System-wide Shift-drag is not claimed. A global hotkey does not provide another process's active drag payload by itself.
- Not release-ready, signed, or fully audited for redistribution licensing.

## Stack
- React + TypeScript + Vite.
- Tauri 2 + Rust.
- Rust zip crate pinned to 8.6.0 with DEFLATE.
- walkdir for directory traversal, tempfile for temporary archives, Tauri dialog plugin for native file selection.
- open crate for opening the destination folder.

## Development
Install Node.js/npm and Rust stable. On Windows, install the prerequisites in the [official Tauri 2 guide](https://v2.tauri.app/start/prerequisites/), including Microsoft C++ Build Tools, Windows SDK and WebView2.

    npm install
    npm run typecheck
    npm run lint
    npm test
    npm run build
    npm run tauri:dev

Build the NSIS installer on Windows with npm run tauri:build.

The browser development server is preview-only. Native paths and ZIP processing require the Tauri desktop build.

## Output and safety
- ZIP destination defaults to %USERPROFILE%/Documents/SHIFT/Archives.
- Source files are read, not modified.
- Outputs use collision-safe names; existing archives are not overwritten.
- Selected symlink roots are rejected; symlinks inside selected folders are skipped.
- Selection is bounded to 1,000 root paths and 100,000 archive entries.
- The temporary archive is reopened and its regular entries are read before the operation reports success.
- Size savings are measured; a ZIP may be larger than its input.
- History is session-only and stores output paths, sizes, entry counts and timestamps, not document contents.
- Core processing has no cloud upload, account, telemetry or backend dependency.

See docs/SECURITY_AND_PERMISSIONS.md, docs/SUPPORTED_FEATURES.md, and docs/RELEASE_CHECKLIST.md.

## Licensing
No project license has been selected yet. Until a license is added, do not assume SHIFT is licensed for redistribution. Review THIRD_PARTY_NOTICES.md and resolve legal placeholders before release.
