# Security and permissions

## Current design
- Core ZIP processing is local; no cloud API, required account, telemetry or backend integration is present.
- The frontend invokes named Tauri commands for app info, ZIP creation and opening the destination folder; no arbitrary shell bridge is exposed.
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
- No global keyboard hook, startup registration, file association or Explorer extension is installed.
- System-wide Shift-drag is not implemented.

## Remaining review
- Test Windows path and permission behavior, invalid inputs, Unicode, large files, locked files, disk-full conditions, monitor/DPI changes and Explorer extraction.
- Review Tauri capabilities against resolved package versions.
- Add bounded/redacted diagnostics before integrating external processes.
- Complete dependency/license inventory and verify network behavior of packaged dependencies.
- Do not add archive extraction before path traversal, symlink, expansion-size, depth/count and collision defenses are tested.
