# Third-party notices

This is a provisional direct-dependency and optional-engine inventory, not a full transitive license audit. Reconcile it against committed npm/Cargo lockfiles and shipped artifacts before distribution.

| Component | Usage | License / official reference | Packaging |
|---|---|---|---|
| React / React DOM | UI runtime | MIT — https://github.com/facebook/react | Direct npm dependencies |
| Vite | Frontend build | MIT — https://github.com/vitejs/vite | Direct npm dev dependency |
| TypeScript | Type-checking | Apache-2.0 — https://github.com/microsoft/TypeScript | Direct npm dev dependency |
| Tauri 2 / API / plugins | Desktop shell/native APIs | Review exact package/crate licenses — https://github.com/tauri-apps/tauri | Direct dependencies |
| Lucide React | Icons | ISC — https://github.com/lucide-icons/lucide | Direct npm dependency |
| Rust zip crate 8.6.0 | ZIP creation/read-back | MIT — https://github.com/zip-rs/zip2 | Direct Cargo dependency |
| image crate | Raster decode/encode | MIT OR Apache-2.0 — https://github.com/image-rs/image | Direct Cargo dependency; codecs are feature-gated |
| flate2 | GZIP encode/decode | MIT OR Apache-2.0 — https://github.com/rust-lang/flate2 | Direct Cargo dependency |
| tar crate | TAR archive creation/extraction | MIT OR Apache-2.0 — https://github.com/alexcrichton/tar-rs | Direct Cargo dependency |
| walkdir / tempfile / open | Traversal, staging and opening destinations | Verify exact resolved release notices — https://github.com/BurntSushi/walkdir ; https://github.com/Stebalien/tempfile ; https://github.com/Byron/open-rs | Direct Cargo dependencies |
| FFmpeg | Optional media codecs/transcoding | License depends on the specific FFmpeg build/configuration — https://ffmpeg.org/legal.html | Not bundled by default; user-installed local engine |
| LibreOffice | Optional office/document conversion | https://www.libreoffice.org/about-us/licenses/ | Not bundled by default |
| qpdf | Optional PDF structural tools | https://qpdf.sourceforge.io/ | Not bundled by default |
| Poppler tools | Optional PDF rendering/text extraction | https://poppler.freedesktop.org/ | Not bundled by default |
| 7-Zip | Optional RAR/7Z handling | https://www.7-zip.org/ | Not bundled by default |
| Inkscape | Optional SVG rendering | https://inkscape.org/release/ | Not bundled by default |

## Before redistribution

1. Commit and inspect `package-lock.json` and `src-tauri/Cargo.lock`.
2. Inventory exact direct and transitive dependency versions, license texts and attribution requirements.
3. Review codecs enabled in any FFmpeg build that is redistributed; GPL/nonfree configurations can change distribution obligations.
4. If packaging optional engines later, review each engine's redistribution obligations and include all required notices.
5. Choose and add SHIFT's own project license.
