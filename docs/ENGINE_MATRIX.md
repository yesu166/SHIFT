# Engine matrix

Engine selection depends on actual capability, exact version, license, notices, provenance, security behavior and redistributable footprint. Items marked planned are not connected to the UI.

| Purpose | Engine | Version / source | License and packaging | Status |
|---|---|---|---|---|
| ZIP creation and read-back validation | Rust zip crate | Cargo.toml pins 8.6.0; https://github.com/zip-rs/zip2 | Upstream identifies MIT; exact crate notice and transitive dependencies still require review | IMPLEMENTED in source; CI results pending |
| Directory traversal | walkdir | Cargo.toml constraint 2.5.0; https://github.com/BurntSushi/walkdir | Resolve and audit the exact lockfile version | IMPLEMENTED in source |
| Temporary output | tempfile | Cargo.toml constraint 3.27.0; https://github.com/Stebalien/tempfile | Resolve and audit exact crate/license | IMPLEMENTED in source |
| Open output folder | open | Cargo.toml constraint 5.3.2; https://github.com/Byron/open-rs | Resolve and audit exact crate/license | IMPLEMENTED in source |
| PDF structural operations | qpdf | https://github.com/qpdf/qpdf and https://qpdf.readthedocs.io/ | Upstream identifies Apache-2.0; verify chosen release and notices | PLANNED |
| PDF rendering | PDFium or alternative | https://pdfium.googlesource.com/pdfium.git | Verify build provenance, license/NOTICE, architecture and redistribution terms | PLANNED |
| Office-to-PDF conversion | LibreOffice | https://www.libreoffice.org/licenses/ | Detect a user-installed dependency first; no silent installation; review redistribution before bundling | PLANNED |
| PDF watermarks/page numbers | To be selected | — | Requires a verified PDF-writing API, not only a renderer | PLANNED |

## ZIP implementation details
- DEFLATE compression at level 6.
- Streams source bytes instead of loading entire files into memory.
- Files, nested directories and empty directory entries.
- Rejects selected symbolic-link roots and skips symbolic links found during folder traversal.
- Rejects overlapping selections and duplicate archive entry names.
- Limits selection to 1,000 root paths and 100,000 archive entries.
- Uses a temporary file in the output directory; reparses the archive and reads all regular entries before finalization.
- Measures actual input/output bytes; the archive can be larger than the inputs.
- Does not intentionally overwrite an existing output.

Windows CI now includes an independent `Expand-Archive` extraction test with a Unicode filename and byte-for-byte content verification. Windows Explorer GUI verification, ZIP64 stress tests, and large-file stress tests remain release gates.
