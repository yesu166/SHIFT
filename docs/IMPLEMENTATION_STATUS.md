# Implementation status

Last reviewed: 2026-10-18

## Repository and environment
- Repository: yesu166/SHIFT, default branch main; implementation branch feat/shift-windows-app.
- The repository was empty before the documentation bootstrap.
- The implementation is committed through the GitHub API. No local Windows desktop session is available here.
- GitHub Actions has executed the initial CI run; it exposed a TypeScript parse error and Rust formatting differences. Those are being corrected, and a passing run is required before claiming tests pass.
- No Windows installer or standalone Windows runtime test is claimed.

## Feature status

| Area | Status | Evidence / next action |
|---|---|---|
| React/Vite application source | IMPLEMENTED | Frontend source committed to the feature branch; rerun typecheck/lint/unit tests/build. |
| Theme and reduced-motion settings | IMPLEMENTED | User settings affect CSS and persist in localStorage. |
| Typed operation registry | IMPLEMENTED | src/lib/operations.ts is shared by workspace filtering and the quick-action wheel. |
| In-app quick-action wheel | IMPLEMENTED | Shows only registry actions marked available; Escape closes it. |
| Native file picker and app-window drop | IMPLEMENTED | Uses Tauri dialog API and the typed webview drag/drop events; Windows runtime verification remains pending. |
| ZIP creation engine | IMPLEMENTED | Rust source streams files using DEFLATE, writes a temporary output, validates entries, measures size, and avoids replacing existing archives. |
| ZIP unit tests | TESTED | Prior Windows CI run passed all five core ZIP tests and Clippy. A Windows-only independent PowerShell `Expand-Archive` interoperability test has been added; its result is pending the current CI run. |
| ZIP output validated with Windows Explorer | BLOCKED | Requires independent Windows-compatible extraction verification. |
| Frontend typecheck/lint/test/build | IN_PROGRESS | The first CI run exposed a frontend parse error and Rust format differences. The follow-up exposed additional TypeScript typing issues; these are being corrected with each CI cycle. |
| PDF operations | PLANNED | Evaluate qpdf and a PDF-writing engine, then add real-output tests. |
| Office-to-PDF conversion | PLANNED | Detect optional LibreOffice and test document layout before enabling conversion. |
| Cross-app Shift-drag | BLOCKED | Must prove a documented, reliable Windows method for obtaining the active drag payload. |
| Windows installer | IN_PROGRESS | CI now attempts an unsigned Windows x64 NSIS package, records file size/SHA-256, and uploads the installer artifact; actual installation/uninstallation and user-machine verification remain required. |
| Lockfiles and full license inventory | IN_PROGRESS | package-lock.json and src-tauri/Cargo.lock are committed by CI (commit bc576c6); npm audit reported 0 vulnerabilities after upgrading Vitest. Complete transitive license/NOTICE audit remains outstanding. |

## Implementation notes
- create_zip executes on Tauri's blocking task pool.
- Output directory is the user's Documents folder joined with SHIFT/Archives.
- Output filenames use collision-safe numbered suffixes.
- Browser preview does not treat browser File objects as native paths; ZIP execution is enabled only in Tauri with native paths.
- Planned operations remain unavailable instead of producing fake outputs.
- ZIP creation rejects overlapping selections; input folders and their child files cannot both be selected into one archive job.

## Next executable tasks
1. Rerun CI and fix every real frontend/Rust/test failure.
2. Generate and commit package-lock.json and src-tauri/Cargo.lock.
3. Independently extract generated ZIP output on Windows and verify bytes/content.
4. Build and inspect the NSIS package; test first launch and uninstall output preservation.
5. Evaluate qpdf and LibreOffice integrations after the ZIP workflow passes.


## Security follow-up
The initial dependency audit identified three npm advisories in the Vitest 3 resolution (one moderate and two critical, involving @vitest/mocker and tinypool). The declared Vitest range was upgraded to ^5.0.5; the latest frontend audit reports 0 vulnerabilities and CI fails if a later npm audit reports any advisory.
