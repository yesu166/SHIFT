# Implementation status

Last reviewed: 2026-10-19

## Repository and environment
- Repository: yesu166/SHIFT, default branch main; implementation branch feat/shift-windows-app.
- The repository was empty before the documentation bootstrap.
- The implementation is committed through the GitHub API. No local Windows desktop session is available here.
- GitHub Actions has executed the initial CI run; it exposed a TypeScript parse error and Rust formatting differences. Those are being corrected, and a passing run is required before claiming tests pass.
- No manual Windows desktop GUI test is claimed; the packaged installer must pass CI and then be tested interactively.

## Feature status

| Area | Status | Evidence / next action |
|---|---|---|
| React/Vite application source | IMPLEMENTED | Frontend source committed to the feature branch; rerun typecheck/lint/unit tests/build. |
| Theme and reduced-motion settings | IMPLEMENTED | User settings affect CSS and persist in localStorage. |
| Typed operation registry | IMPLEMENTED | src/lib/operations.ts is shared by workspace filtering and the quick-action wheel. |
| Native overlay wheel and global shortcut | IMPLEMENTED IN SOURCE | Separate transparent always-on-top wheel window; F6–F12 configurable (F8 default); manual Windows GUI testing remains required. |
| Native file picker and app-window drop | IMPLEMENTED | Uses the Tauri dialog API and typed webview drag/drop events; Windows runtime verification remains pending. |
| ZIP creation engine | IMPLEMENTED | Rust source streams files using DEFLATE, writes a temporary output, validates entries, measures size, and avoids replacing existing archives. |
| ZIP unit tests | TESTED | Prior Windows CI run passed all five core ZIP tests and Clippy. A Windows-only independent PowerShell `Expand-Archive` interoperability test has been added; its result is pending the current CI run. |
| ZIP output validated with Windows Explorer | BLOCKED | Requires independent Windows-compatible extraction verification. |
| Frontend typecheck/lint/test/build | PASS ON LATEST OVERLAY REVISION | Typecheck, lint, unit tests, production build and npm audit passed. See the latest GitHub Actions run for current native-build status. |
| PDF operations | PLANNED | Evaluate qpdf and a PDF-writing engine, then add real-output tests. |
| Office-to-PDF conversion | PLANNED | Detect optional LibreOffice and test document layout before enabling conversion. |
| Cross-app drag-to-wheel | IMPLEMENTED IN SOURCE; MANUAL TEST REQUIRED | The separate overlay subscribes to Tauri's native drag/drop event and routes a drop on Compress to ZIP into the Rust ZIP engine. Verify Explorer drag continuity and focus behavior on Windows. |
| Windows installer | CI / MANUAL VERIFICATION REQUIRED | CI attempts an unsigned Windows x64 NSIS package and uploads it as an artifact; install/uninstall behavior still needs an interactive Windows test. |
| Lockfiles and full license inventory | IN_PROGRESS | package-lock.json and src-tauri/Cargo.lock are committed by CI (commit bc576c6); npm audit reported 0 vulnerabilities after upgrading Vitest. Complete transitive license/NOTICE audit remains outstanding. |

## Implementation notes
- create_zip executes on Tauri's blocking task pool.
- Output directory is the user's Documents folder joined with SHIFT/Archives.
- Output filenames use collision-safe numbered suffixes.
- Browser preview does not treat browser File objects as native paths; ZIP execution is enabled only in Tauri with native paths.
- Planned operations remain unavailable instead of producing fake outputs.
- ZIP creation rejects overlapping selections; input folders and their child files cannot both be selected into one archive job.

## Next executable tasks
1. Pass the latest Windows Rust test, Clippy, and NSIS installer CI jobs.
2. Verify Explorer drag/drop, focus preservation, Escape, tray close/quit, and DPI/multi-monitor behavior in a real Windows desktop session.
3. Independently extract generated ZIP output on Windows and verify bytes/content.
4. Test first launch and uninstall behavior while preserving user archives.
5. Evaluate PDF and LibreOffice integrations after the ZIP workflow passes.


## Security follow-up
The initial dependency audit identified three npm advisories in the Vitest 3 resolution (one moderate and two critical, involving @vitest/mocker and tinypool). The declared Vitest range was upgraded to ^5.0.5; the latest frontend audit reports 0 vulnerabilities and CI fails if a later npm audit reports any advisory.
