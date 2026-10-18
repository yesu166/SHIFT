# SHIFT

**A Windows-first, local-first document toolbox.**

SHIFT is planned as a native Windows desktop app built with **React, TypeScript, Vite, Tauri 2, and Rust**. Its goal is to make common document actions—conversion, PDF organization, compression, protection, and batch processing—fast and understandable.

> **Project status:** Initial repository bootstrap. No application workflows are implemented or verified yet. This README will be updated as working features and tests are delivered.

## Goals

- Windows 11 x64 first.
- Process files locally by default; no required account, cloud upload, telemetry, or backend.
- Preserve original files and avoid silent overwrites.
- Offer only operations whose engine support and output validation have been verified.
- Provide an accessible, keyboard-friendly interface with a compact radial action menu.
- Keep processing and file operations in the Rust/Tauri layer, with narrow, explicit permissions.

## Planned stack

- React + TypeScript + Vite — UI and frontend build.
- Tauri 2 + Rust — desktop shell, native file handling, job orchestration and safe process invocation.
- Rust ZIP support — evaluated for standard compressed archive creation.
- qpdf — evaluated for supported PDF structural operations.
- LibreOffice — optional dependency to evaluate for DOCX/XLSX/PPTX to PDF conversion.

The engine choices above are evaluations, not claims that they are bundled or already working. Engine versions, exact capabilities, licenses, packaging, and tests must be verified before distribution.

## Development status

See [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) for what is implemented, tested, blocked, and next. Status labels distinguish code from tested functionality and Windows-verified behavior.

## Build prerequisites

The desktop build requires a supported Node.js/npm toolchain, Rust/Cargo, and Tauri's documented Windows native build prerequisites, including the appropriate Microsoft C++ Build Tools/Windows SDK and WebView2 environment.

The actual project commands will be documented here once the app scaffold and lockfiles are committed. Consult the official [Tauri 2 Windows setup guide](https://v2.tauri.app/start/prerequisites/) before configuring a Windows development machine.

## Privacy and safety

- Source documents must not be overwritten silently.
- Conversion outputs must be validated before a job reports success.
- Passwords and secrets must never be written to logs or command-line arguments.
- Optional engine installation must be explicit.
- No global keyboard hook, startup behavior, file association, or Explorer integration should be enabled without a justified design and clear user consent.

See [docs/SECURITY_AND_PERMISSIONS.md](docs/SECURITY_AND_PERMISSIONS.md) and [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md).

## License

No project license has been selected yet. Until a license is added, do not assume the source is licensed for reuse or redistribution. Third-party notices will be maintained in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
