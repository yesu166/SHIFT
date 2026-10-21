# SHIFT | Project Brief

> **Proposed repository description:** A planned Windows-first, local-first document toolbox for conversion, PDF organization, compression, and batch processing, using React, Tauri, and Rust.

## Overview
SHIFT is a proposed Windows desktop utility for common document tasks such as conversion, PDF organization, compression, protection, and batch processing. The design prioritizes local processing, clear operations, and preserving the user's original files.

## Problem and opportunity
Document utilities can be confusing about where files are processed, which external engines are required, and whether a result was actually validated. A transparent local-first toolbox can make supported operations and their prerequisites easier to understand.

## Proposed approach
- Use React, TypeScript, and Vite for the interface.
- Use Tauri 2 and Rust for the desktop boundary, file operations, and process orchestration.
- Keep files local by default, with no mandatory account, upload, telemetry, or hosted backend.
- Preserve originals and avoid silent overwrites.
- Offer only operations whose engine support, licensing, and output validation have been tested.
- Keep native permissions narrow and explicit.

## Current scope and maturity
The README labels this repository as an initial bootstrap and states that application workflows are not yet implemented or verified. Engine choices such as qpdf and LibreOffice are under evaluation and should not be described as bundled or working features until validated.

## Roadmap proposal
1. Define the initial operation set and supported input/output formats.
2. Create the smallest functional vertical slice: select a file, run one supported operation, validate output, and report the result.
3. Add file-preservation, cancellation, error reporting, and output-integrity tests.
4. Verify Windows setup, dependency discovery, packaging, and installer behaviour.
5. Document optional engine installation and each engine's limitations and licence.
6. Add batch processing only after single-file operations are reliable.

## Privacy and safety
Never report success before output validation. Do not log passwords or secrets, silently overwrite the source, or enable global keyboard hooks, startup behaviour, and shell integrations without a clear need and user consent.

## Collaboration proposal
At this stage, useful contributions include architecture review, Windows setup documentation, dependency and licence research, output-validation tests, accessibility feedback, and small isolated workflow prototypes. Keep roadmap items labelled as proposed until they are implemented and verified.

## Licensing and reuse
The README states that a project license has not yet been selected. Until a license is added, do not assume the source is licensed for reuse or redistribution. Choose a licence and review third-party engine terms before advertising SHIFT as open source.
