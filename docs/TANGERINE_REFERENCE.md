# Tangerine reference research

Primary product reference: https://tangerineformac.com/
Additional public feature reference: https://www.tangerineformac.com/tools
Windows community reference: https://github.com/CristianO2601/tangerine-windows

## Publicly observable product behavior

The official product site describes an offline Mac file converter with a modifier-driven radial workflow: start dragging a file, press Shift for format conversion, Command for file tools, or Control for file actions. It says the wheel remains open until drop or cancel and describes arrow-key navigation with Enter and Escape. Its published format families include images, audio, video, PDF, documents, slides, spreadsheets, subtitles and archives. It also describes local processing, keeping originals, and a contextual menu that offers only relevant operations. Source: https://tangerineformac.com/

The official website publishes counts of 275 conversion paths, 26 file tools and 6 file actions. Those are Tangerine's published figures, not claims about SHIFT. Source: https://tangerineformac.com/

An independent Windows community project describes a separate Windows implementation using Python/PySide6 plus public Win32 state and the OLE drag clipboard. It is not the original macOS app and carries an MIT license. Source: https://github.com/CristianO2601/tangerine-windows and https://github.com/CristianO2601/tangerine-windows/blob/main/LICENSE

## SHIFT's interpretation

SHIFT implements the underlying interaction pattern in its own Rust/Tauri architecture:
- Shift -> image conversion
- Alt+Shift -> JPEG compression or ZIP tools
- Ctrl+Shift -> copy paths or open the output destination
- outputs are written as new files and sources are preserved
- a typed registry filters the wheel to operations compatible with selected extensions

The first milestone deliberately has fewer engines than Tangerine. Built-in processing covers PNG/JPG/WebP/BMP/TIFF conversion, lossy JPEG re-encoding and validated ZIP creation. PDF, Office and audio/video processing remain unavailable until real local engines and output validation exist.

## What is not known

Tangerine's private source code and internal implementation are not established by the marketing website. This work uses public product behavior and a separately published Windows community project for architectural research; it does not claim access to the original private source code. SHIFT must not copy proprietary branding, artwork, site copy or assets.

## Design and engineering principles for SHIFT

1. Match the low-interruption workflow and contextual radial interaction, not Tangerine's brand or exact artwork.
2. Keep SHIFT's own orange/ink palette, typography, labels and identity.
3. Show only operations supported by selected file types and the engines actually present.
4. Validate generated outputs and preserve originals.
5. Keep core processing local and optional large engines unbundled until size, licenses and reliability are reviewed.
6. Treat Explorer drag/drop, focus behavior, keyboard mode switching, DPI and multi-monitor behavior as manual Windows verification gates.
