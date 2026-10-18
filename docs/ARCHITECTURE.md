# Architecture

## Target architecture

```text
React + TypeScript UI
  ├─ Home / Convert / Organize / Compress / Edit PDF / Protect / Batch & History / Settings
  ├─ Shared typed operation registry (eligibility, options, required engines)
  ├─ Shared job UI and output-validation state
  └─ Narrow Tauri command interface
          ↓
Tauri 2 + Rust
  ├─ Input identification and validation
  ├─ Operation registry enforcement
  ├─ Job queue, progress, cancellation where supported
  ├─ Engine adapters (ZIP, qpdf, optional LibreOffice)
  ├─ Collision-safe output paths and temporary workspaces
  ├─ Output validation and measured byte counts
  ├─ Minimal settings/history
  └─ Windows-specific integration (only documented APIs)
```

This is the target design, not a description of fully implemented modules.

## Boundaries
- UI components render state and submit operation IDs; they do not implement conversion logic.
- The same registry and job orchestration must back dashboard shortcuts, radial actions, and batch operations.
- Rust validates every requested operation and path independently of frontend checks.
- Engine adapters receive discrete arguments and app-owned working directories; do not construct shell command strings.
- Successful process exit is only one signal. Validate outputs by reopening/parsing them and checking expected content or structure.
- Preserve source inputs. Route output to separate SHIFT directories with collision-safe names.
- Use bounded concurrency, bounded diagnostic output, timeouts, and cancellation only where the child engine can actually be stopped.
- Tauri capabilities should grant the smallest required permissions. Never expose arbitrary command execution to the webview.
