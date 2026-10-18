# Engine matrix

No engine is currently bundled or verified in the repository. The items below are candidates to evaluate, not implemented dependencies.

| Purpose | Candidate | Upstream | License status | Packaging model | Status |
|---|---|---|---|---|---|
| ZIP creation | Rust `zip` crate from zip-rs/zip2 project | https://github.com/zip-rs/zip2 | Upstream identifies MIT; verify exact pinned release and all notices | Rust dependency; inspect feature flags and compile impact | PLANNED |
| PDF structural operations | qpdf | https://github.com/qpdf/qpdf | Upstream identifies Apache-2.0; verify chosen release and notices | Optional executable or justified bundle after licensing/size/security review | PLANNED |
| PDF rendering | PDFium or compatible alternative | https://pdfium.googlesource.com/pdfium.git | Must review chosen binary/source license and NOTICE data | Do not bundle before provenance, architecture and redistribution review | PLANNED |
| Office to PDF | LibreOffice | https://www.libreoffice.org/licenses/ | Review exact version, LGPL/MPL and third-party notices / obligations | Prefer explicit installed-dependency detection initially; never download/install silently | PLANNED |
| PDF writing for watermark/page numbers | To be selected | — | Must be checked for exact use and redistribution | Only after proof of supported API and license review | PLANNED |

## Evaluation requirements
Record exact version, source, license/NOTICE, architectures, maintenance status, operations supported, arguments/options, timeout behavior, cancellation behavior, distribution footprint, security considerations, and test results. Free to download does not mean unrestricted redistribution. Avoid adding engines with incompatible license obligations to a commercial binary without deliberate review and approval.
