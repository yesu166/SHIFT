# Third-party notices

This is a provisional inventory of selected direct dependencies. It is not a complete transitive dependency audit. Reconcile it with the actual npm and Cargo lockfiles before releasing.

| Component | Usage | Upstream / license reference | Status |
|---|---|---|---|
| React / React DOM | UI runtime | https://github.com/facebook/react — MIT | Direct npm dependency; exact versions not yet locked |
| Vite | Frontend dev server/build | https://github.com/vitejs/vite — MIT | Direct dev dependency; exact versions not yet locked |
| TypeScript | Type-checking | https://github.com/microsoft/TypeScript — Apache-2.0 | Direct dev dependency; exact version not yet locked |
| Tauri 2 / Tauri API / dialog plugin | Desktop shell and native file selection | https://github.com/tauri-apps/tauri — inspect exact package/crate licenses | Lock versions and include notices |
| Lucide React | Icon components | https://github.com/lucide-icons/lucide — ISC | Lock version and retain license/attribution |
| Rust zip crate 8.6.0 | ZIP creation and read-back | https://github.com/zip-rs/zip2 — MIT | Pinned requirement; verify downloaded license and transitive packages |
| walkdir | Directory traversal | https://github.com/BurntSushi/walkdir | Verify exact resolved crate license |
| tempfile | Temporary archive files | https://github.com/Stebalien/tempfile | Verify exact resolved crate license |
| open | Open destination folder | https://github.com/Byron/open-rs | Verify exact resolved crate license |

## Before distribution
1. Generate and commit package-lock.json and src-tauri/Cargo.lock.
2. Inventory all resolved direct/transitive dependencies and any bundled executable.
3. Include exact required license texts and notices for shipped packages.
4. Review redistribution obligations for any document engine; free downloads do not mean unrestricted redistribution.
5. Choose and add SHIFT's own project license before inviting external reuse.
