# SHIFT

SHIFT is a Windows-first, local-first document toolbox built with React, TypeScript, Vite, Tauri 2 and Rust.

The first workflow under implementation is **Compress files to ZIP**: select files or folders, stream them into a DEFLATE archive, reopen and read each regular-file entry for validation, measure actual input/output sizes, and save under Documents/SHIFT/Archives without replacing an existing output.

## Current status

- Implemented in source: React workspace, shared typed operation registry, quick-action wheel, light/dark theme, reduced-motion setting, native file selection/drop bridge, session history, and Rust ZIP engine.
- CI is active and must pass before tests are considered verified. See [SHIFT CI](.github/workflows/ci.yml).
- Not yet verified: packaged Windows app, installer, cross-monitor/DPI behavior, independent Windows Explorer extraction.
- PDF operations, password protection and Office conversion are deliberately unavailable until real engines and output tests are added.
- System-wide Shift-drag is not claimed: a global hotkey does not provide another process's drag payload by itself.
- Project is not yet release-ready, signed, or fully audited for redistribution licensing.

## Stack

- React + TypeScript + Vite.
- Tauri 2 + Rust.
- Rust ZIP crate pinned to 8.6.0 with the DEFLATE feature.
- walkdir for directory traversal; tempfile for temporary archives; Tauri dialog plugin for native file selection.
- open crate to open the output directory in the system file manager.

## Development

Install a supported Node/npm version and Rust stable. On Windows, install the prerequisites in the [official Tauri 2 guide](https://v2.tauri.app/start/prerequisites/), including Microsoft C++ Build Tools, Windows SDK, and WebView2.

    npm install
    npm run typecheck
    npm run lint
    npm test
    npm run build
    npm run tauri:dev

Build the NSIS installer on Windows with npm run tauri:build.

The browser development server is preview-only. Native file paths and ZIP processing require the Tauri desktop build.

## Output and safety

- ZIP archives default to %USERPROFILE%/Documents/SHIFT/Archives.
- Source files are read, not modified.
- Outputs are collision-safe; existing archives are not overwritten.
- Selected symlink roots are rejected; symlinks within chosen folders are skipped.
- Directory traversal is bounded to 1,000 selected roots and 100,000 archive entries.
- Temporary output is written in the destination folder, reopened, and all regular entries are read before the archive is finalized.
- The app measures real size changes. A ZIP may be larger than its inputs.
- History is session-only and stores output paths, sizes, entry counts and timestamps—not file contents.
- Core processing has no cloud upload, required account, telemetry, or backend service.

See [docs/SECURITY_AND_PERMISSIONS.md](docs/SECURITY_AND_PERMISSIONS.md), [docs/SUPPORTED_FEATURES.md](docs/SUPPORTED_FEATURES.md), and [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md).

## Licensing

No project license has been selected yet. Until a license is added, do not assume SHIFT source is licensed for redistribution. Review THIRD_PARTY_NOTICES.md and resolve the legal placeholders before release.
