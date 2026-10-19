# Security and permissions

## Current design
- Core ZIP processing is local; no cloud API, required account, telemetry or backend integration is present.
- The frontend invokes named Tauri commands for app info, ZIP creation, opening the destination folder, showing/hiding the wheel, and reading/changing its shortcut; no arbitrary shell bridge is exposed.
- File selection uses the Tauri dialog plugin; OS file drops use the typed Tauri webview event API.
- ZIP work runs in a blocking worker.
- Selected symbolic-link roots are rejected; symbolic links inside selected folders are skipped.
- Duplicate and overlapping input roots are rejected.
- The archive is written to a temporary file in Documents/SHIFT/Archives, reopened, and every regular entry is read before it is finalized.
- Final output uses a unique path and non-clobbering persistence; existing archives are not intentionally overwritten.
- Archive-name input rejects control characters, reserved filename characters/device names, path separators, and trailing dots/spaces.
- Enumeration is bounded to 1,000 selected roots and 100,000 entries.
- Session history contains output path, sizes, entry count and timestamp, not document contents or passwords.

## Native permissions
- The Tauri capability grants core defaults and dialog open permission.
- The output folder opens through the Rust open crate.
- A global function-key shortcut (F6–F12; F8 default) is registered via the Tauri global-shortcut plugin. Escape is registered only while the wheel is visible and is removed when it hides.
- The separate wheel window is intended to be non-focusable so it does not steal focus from Explorer during a drag; this behavior still needs a manual Windows test.
- Closing the main window hides it to the notification-area tray. The tray provides Open SHIFT and Quit SHIFT. No Windows Service Control Manager service, sign-in autostart, file association, or Explorer extension is installed.
- Cross-app drag/drop is implemented in source using native Tauri events; OLE drag continuity and target hit testing remain manual verification gates.

## Remaining review
- Test Windows path and permission behavior, invalid inputs, Unicode, large files, locked files, disk-full conditions, monitor/DPI changes and Explorer extraction.
- Review Tauri capabilities against resolved package versions.
- Add bounded/redacted diagnostics before integrating external processes.
- Complete dependency/license inventory and verify network behavior of packaged dependencies.
- Do not add archive extraction before path traversal, symlink, expansion-size, depth/count and collision defenses are tested.
