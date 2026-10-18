# SHIFT development rules

## Fixed stack
- Use React, TypeScript, Vite, Tauri 2, and Rust.
- Do not switch to Electron, WinUI, C#, or a separate C++ backend without a demonstrated technical blocker and explicit approval.
- Inspect the current branch and diff before editing. Preserve unrelated work.

## Correctness
- Do not call a feature working unless its real output has been checked.
- Never overwrite inputs silently. Use collision-safe output paths.
- A zero exit code or output-file existence alone is not sufficient; validate the produced document/archive.
- Unsupported or unavailable operations must be hidden or disabled with an honest explanation.
- Keep secrets and document contents out of logs and commits.
- Do not make network calls for document operations unless the user explicitly opts into a documented network feature.
- Use narrow Tauri commands and capabilities. Do not expose arbitrary shell execution or unrestricted filesystem access to the frontend.

## Engineering loop
1. Inspect one concrete area.
2. Implement a small coherent feature.
3. Run relevant tests/builds and inspect actual outputs.
4. Fix failures rather than hiding them.
5. Update `docs/IMPLEMENTATION_STATUS.md` and relevant technical docs.
6. Review the diff and commit coherent milestones only.

## Status vocabulary
Use only: `PLANNED`, `IN_PROGRESS`, `IMPLEMENTED`, `TESTED`, `VERIFIED_ON_WINDOWS`, `BLOCKED`, `NOT_SUPPORTED`.
- `IMPLEMENTED` means code exists.
- `TESTED` means the relevant tests were run and passed.
- `VERIFIED_ON_WINDOWS` requires execution on Windows.
- Mark a task blocked with the exact blocker and a next action.

## Commands
Document real project commands in README only when the scaffold provides them. Do not invent successful command output or claim CI/release artifacts exist before verification.
