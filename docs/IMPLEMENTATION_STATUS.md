# Implementation status

Last reviewed: 2026-10-19

## Architecture
- React/TypeScript/Vite renders the main workspace and a separate transparent, always-on-top wheel.
- Tauri 2 + Rust owns OS integration and processing. The packaged app does not need a separate HTTP service.
- A Windows polling thread watches for a real mouse drag, movement and modifier combination from Explorer/Desktop. It shows the overlay without explicitly focusing it and emits the current wheel mode; Tauri native file-drop events deliver selected paths to the webview.
- The central typed operation registry in src/lib/operations.ts filters actions by source extensions and mode.

## Feature status

| Area | Status | Evidence / next action |
|---|---|---|
| Contextual modifier-driven wheel | IMPLEMENTED IN SOURCE | Shift=convert, Alt+Shift=tools, Ctrl+Shift=file actions. Explorer drag, mode switching, focus and DPI require manual desktop verification. |
| Operation registry | IMPLEMENTED | src/lib/operations.ts drives menu filtering and unit tests. |
| ZIP creation | IMPLEMENTED IN SOURCE | Rust streams DEFLATE level 6 into temporary output, reads entries back, measures size and avoids overwrites. CI plus manual Explorer extraction verification required. |
| Image conversion | IMPLEMENTED IN SOURCE | Rust image crate supports PNG/JPG/WebP/BMP/TIFF; temporary output is decoded again, then saved with a collision-safe name beside the original. |
| JPEG compression | IMPLEMENTED IN SOURCE | Rust JPEG encoder quality 78; source preserved, output size may increase. |
| Copy paths / open output location | IMPLEMENTED IN SOURCE | Clipboard bridge and native folder opening; manual tests required. |
| Tray behavior | IMPLEMENTED IN SOURCE | Main window close hides to tray; quit menu exits. Validate lifecycle manually. |
| PDF operations | PLANNED | No PDF engine connected. |
| Office conversion | PLANNED | No Office engine connected. |
| Audio/video conversion | PLANNED | No FFmpeg integration connected. |
| Windows installer | CI / MANUAL VERIFICATION REQUIRED | Build in Windows CI; test install, launch, uninstall and data preservation interactively. |
| Project license and dependency audit | IN PROGRESS | Review image crate and resolved transitive dependencies, include exact notices and choose SHIFT's own license before distribution. |

## Manual validation checklist
1. Drag image(s) from Explorer, then tap Shift while holding the mouse button. Confirm the wheel appears at the cursor.
2. Drag onto each format; verify output files are valid, source files unchanged and existing output names not overwritten.
3. Hold Alt+Shift to see JPEG compression (JPEG selections only) or Create ZIP; inspect ZIP using Windows Explorer.
4. Hold Ctrl+Shift and test Copy paths and Output folder.
5. Drop outside the wheel, press Escape, and close the main window; confirm cancellation, tray reopen and quit behavior.
6. Repeat at 100%, 150% and 200% display scaling and with multiple monitors.
7. Verify behavior when the key combination conflicts with another application and when selection contains unsupported types.

## Known boundaries
- The drag monitor is restricted to common Explorer/Desktop window classes; arbitrary application-to-application drags are not promised yet.
- The current image engine does not promise animated image conversion, HEIC/AVIF/SVG input or metadata fidelity.
- ZIP archiving is a container/bundling operation, not a promise to shrink every file. The UI displays measured byte differences.
- Audio/video, PDF and Office conversions are not implemented. The wheel must not expose fake actions for them.
