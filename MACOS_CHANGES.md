# MacUtl — macOS fork changes

- **Upstream**: NeoUtl v0.8.11, commit `0ead39350db55996810de53da79d02fbc98355f5`
  (<https://codeberg.org/taisho-guy/NeoUtl>)
- **Fork**: MacUtl 0.8.11 (macOS fork; Phase 7)
- **Change date**: 2026-11-12 (initial release), 2026-11-13 (Phase 7)
- **License**: GNU Affero General Public License, version 3 or later
  (see `LICENSE`; upstream copyright notices are retained)

This document records the changes made in the macOS fork, as required by the
AGPL when conveying a modified version. The fork is unofficial and is not
affiliated with the upstream NeoUtl project or with AviUtl.

## 1. Scope of the fork

MacUtl is a macOS-native port/fork of NeoUtl. The upstream project's history,
copyright notices and license are preserved. The fork adds macOS platform
support, packaging, and the product-name changes described below. Internal
crate names, the `.npb` project format and source module names remain
unchanged for compatibility and upstream tracking.

## 2. macOS platform changes (Phase 0–3)

### 2.1 Build and runtime

- Added macOS build support for the workspace and plugin crates
  (`aarch64-apple-darwin`).
- Added `.app`-aware runtime data location:
  `~/Library/Application Support/MacUtl` when running inside
  `Contents/MacOS`; bare development binaries keep the executable-adjacent
  layout.
- Added macOS defaults for the VST3 plugin search path and plugin-host lookup
  next to the main executable.
- Disabled the in-app self-updater on macOS. The updater would replace the
  binary and invalidate the code signature; updates are delivered manually via
  GitHub Releases.
- Disabled the upstream crash-reporting client on macOS at compile time. The
  upstream GlitchTip DSN is not compiled into the macOS binary.

### 2.2 Rendering, media and export

- Added VideoToolbox hardware decoding with software fallback.
- Added shared wgpu device/queue handling and macOS-specific limits / NV12
  capability negotiation.
- Added macOS Hiragino Sans fallback for text rendering.
- Added Japanese IME preedit/commit handling and input-rectangle notification
  in the property editor.
- Added direct output-path editing in the export dialog and a macOS default
  file name.
- Added export error diagnostics for `avio_open` failures.

### 2.3 Phase 4.5 export-hang fix

- Fixed a lost-wakeup deadlock when a video object and an image object were
  exported together. The image decoder dylib statically links its own copy of
  wgpu/parking_lot; its texture upload could race the host's queue
  `submit`/`map_async`.
- On macOS only, serialized image `texture()` uploads, decoder `map_async`,
  and `write_buffer` with the host's shared wgpu lock. Hardware decoding
  remains enabled.
- Verified with the Phase 2 video+image project: 391 frames complete,
  720p; 1080p/60 s export remains ~37 s (no regression).

## 3. Packaging and product identity (Phase 4–5)

- Added `scripts/package_macos.py` and `scripts/package_macos.sh`, plus
  `cargo xtask package --release`.
- The packager:
  - copies the release binary and all object/effect/decoder plugins;
  - recursively collects Homebrew dylibs into `Contents/Frameworks`;
  - rewrites install names to `@rpath` and verifies no Homebrew absolute paths
    remain;
  - generates `Info.plist`, `PkgInfo`, and the `.icns` icon;
  - copies `LICENSE`, `THIRD_PARTY_NOTICES.md`, `MACOS_CHANGES.md`, and
    `third_party/licenses/` into `Contents/Resources`;
  - applies an ad-hoc signature (`codesign --sign -`) from the inside out;
  - creates `MacUtl-0.8.11-macos.dmg` and `.zip`.
- Product name changed to **MacUtl** for user-visible surfaces:
  - `.app` name: `MacUtl.app`;
  - `CFBundleName` / `CFBundleDisplayName`: `MacUtl`;
  - `CFBundleIdentifier`: `works.rayno.macutl`;
  - main executable inside the bundle: `MacUtl`;
  - window titles: `MacUtl` / `MacUtl - <project>`;
  - runtime data: `~/Library/Application Support/MacUtl`.
- Internal crate names (`NeoUtl`, `neoutl-*`), the `.npb` format, and source
  module names are unchanged. "NeoUtl" remains in credits, upstream references,
  and internal identifiers by design.
- The upstream README is preserved as `README.upstream.md`; `README.md` is the
  MacUtl fork README.

## 4. One-screen workspace (Phase 7)

- Added `egui_tiles` 0.17.1 and `src/app/workspace.rs`.
- Preview, properties and timeline now share one window; the separate
  timeline/properties viewports were removed.
- Added YMM4-style and classic presets, splitter resizing, pane-header drag
  swapping, header context-menu swapping, and layout reset.
- The layout is persisted as JSON outside the `.npb` project and
  `SystemSettingsResource`, at
  `~/Library/Application Support/MacUtl/settings/workspace-layout.json`
  (development builds: `<executable dir>/settings/workspace-layout.json`).
- Invalid or corrupt layout JSON falls back to the YMM4-style preset.
- Added a minimum window size of 800×600 and small-panel guards in the
  timeline and properties panels.
- Narrow effect lists now omit the status badge or truncate long effect
  names so the label and badge do not overlap.
- Moved the playback tick from the preview `show` call to the app level.
- Playback controls moved below the preview.
- The existing dialogs remain separate windows in v1.

## 5. License and notices

- Added `LICENSE_AUDIT.md` (license audit and publication gate).
- Added `THIRD_PARTY_NOTICES.md` (508 Rust license entries plus native
  libraries and non-code assets).
- Added `third_party/licenses/` with the full texts for FFmpeg (GPLv3/LGPL),
  x264, x265, SVT-AV1, dav1d, libvpx, libvmaf, OpenSSL, LAME, mpg123, Opus and
  XZ/liblzma.
- Added `about.toml`, `scripts/about.hbs`, and
  `scripts/generate_third_party_notices.sh` to make the Rust notices
  reproducible.
- The upstream `LICENSE` and copyright notices are retained unchanged.

## 6. Verification records

The following reports are part of the repository and document the work:

- `MACOS_PHASE0_1_REPORT.md`
- `MACOS_PHASE2_REPORT.md`
- `MACOS_PHASE3_REPORT.md`
- `MACOS_PHASE4_REPORT.md`
- `MACOS_PHASE45_REPORT.md`
- `MACOS_PHASE5_REPORT.md`
- `MACOS_PHASE7_REPORT.md`

Key verification results are summarized in `MACOS_PHASE5_REPORT.md`
(regression checks, release build, downloaded-artifact verification).

## 7. Build-time generated file

`cargo xtask build` regenerates `i18n/ja.yml` from Japanese string literals in
the source tree (existing upstream behavior). The generated file belongs to
the `i18n` submodule's working tree; it is not committed to the MacUtl
superproject. A clean build reproduces it automatically. For a checkout that
must exactly match the release build without running xtask, apply
`patches/i18n-macutl-ja.patch` to the `i18n` submodule first.

## 8. Contact

- Issues: <https://github.com/Ren9618/MacUtl/issues>
- Upstream: <https://codeberg.org/taisho-guy/NeoUtl>

MacUtl is an unofficial fork. It is not the official AviUtl or NeoUtl, and it
is not affiliated with or endorsed by their authors.
