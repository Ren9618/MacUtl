# NeoUtl macOS 移植 Phase 0（調査・ブロッカー棚卸し・ライセンス一次確認）& Phase 1（最小起動確認）報告書

作成日: 2026-11-12  
対象作業ディレクトリ: `/Users/ray_9618/Desktop/Git/NeoUtl-macOS`

---

## 1. 環境の実測値（0-1）とクローン元・HEAD の実測値

### 1.1 マシン・OS・ハードウェア実測値

| 項目 | 実行コマンド | 実測結果 |
|---|---|---|
| macOS バージョン | `sw_vers` | `ProductName: macOS` / `ProductVersion: 27.0.1` / `BuildVersion: 26A434` |
| CPU アーキテクチャ | `uname -m` | `arm64` |
| Mac モデル / CPU | `sysctl -n hw.model machdep.cpu.brand_string` | `MacBookAir10,1` / `Apple M1` |
| コア数 / 物理メモリ | `sysctl -n hw.ncpu hw.perflevel0.logicalcpu hw.perflevel1.logicalcpu hw.memsize` | 論理 8 コア（高性能 4 コア + 高効率 4 コア）/ `17179869184` bytes（16 GB RAM） |

### 1.2 ツールチェーン実測値（作業開始時点 → ユーザー領域への追加導入後）

| 項目 | 作業開始時点の実測値 | 作業中にユーザー領域へ導入したもの・導入後の実測値 |
|---|---|---|
| Active Developer Directory | `/Library/Developer/CommandLineTools`（`xcode-select -p`） | 変更なし（`sudo` 不使用のためシステム設定変更なし） |
| Xcode | `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -version` → `Xcode 27.2 Build version 27B52`（素の `xcodebuild -version` は CLT が active のためエラー終了） | 変更なし |
| Apple Clang | `Apple clang version 21.0.0 (clang-2100.3.34.2)` / `Target: arm64-apple-darwin27.0.0` | 変更なし |
| Rust (`rustc` / `cargo` / `rustup`) | 未導入（`command not found`） | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- -y --no-modify-path` により `~/.cargo` / `~/.rustup` へ導入:<br>- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`<br>- `cargo 1.99.1 (5f94df478 2026-08-27)`<br>- `rustup 1.29.3 (edbc831ce 2026-11-02)` |
| Homebrew | `Homebrew 7.0.12`（`/opt/homebrew/bin/brew`） | 変更なし |
| Homebrew パッケージ | 導入済み: `ffmpeg 9.0.2`, `ffmpeg-full 9.0.2`<br>未導入: `luajit`, `pkg-config`(`pkgconf`), `cmake`, `slang` | `brew install pkgconf` により **`pkgconf 3.0.10`**（`/opt/homebrew/bin/pkg-config`）をユーザー領域に導入（`ffmpeg-sys-next` のビルドに必須）。`luajit` は `mlua` の `vendored` 機能でソースビルドされ、`slang` は `xtask` が `slang/bin/slangc`（`v2026.22`）を自動取得するため Homebrew への追加導入不要 |

#### `ffmpeg -version` 実測出力（抜粋ではなく全文）

```text
ffmpeg version 9.0.2 Copyright (c) 2000-2026 the FFmpeg developers
built with Apple clang version 21.0.0 (clang-2100.3.34.2)
configuration: --prefix=/opt/homebrew/Cellar/ffmpeg/9.0.2 --enable-shared --enable-pthreads --enable-version3 --cc=clang --host-cflags= --host-ldflags= --enable-ffplay --enable-gpl --enable-libsvtav1 --enable-libopus --enable-libx264 --enable-libmp3lame --enable-libdav1d --enable-libvmaf --enable-libvpx --enable-libx265 --enable-openssl --enable-videotoolbox --enable-audiotoolbox --enable-neon
libavutil      61.  1.102 / 61.  1.102
libavcodec     63.  1.102 / 63.  1.102
libavformat    63.  1.102 / 63.  1.102
libavdevice    63.  1.102 / 63.  1.102
libavfilter    12.  1.102 / 12.  1.102
libswscale     10.  1.102 / 10.  1.102
libswresample   7.  1.102 /  7.  1.102
```

### 1.3 クローン元・HEAD・Submodule の実測値

- **クローン実行コマンド**: `git clone https://codeberg.org/taisho-guy/NeoUtl.git ~/Desktop/Git/NeoUtl-macOS`
  - Codeberg 側で `https://codeberg.org/TeamNeoUtl/NeoUtl/` へ自動リダイレクトされた。
- **クローン直後の `git rev-parse HEAD` 実測値**:
  - `0ead39350db55996810de53da79d02fbc98355f5`（計画書記載のコミットハッシュと完全一致）
  - コミット情報: `2026-11-11 23:29:47 +0900` (`feat(renderer): Framebuffer ObjectをClip / Groupの2種に分解`)
- **Submodule 初期化（`git submodule update --init --recursive`）の実測結果**:
  - `assets` (`https://codeberg.org/TeamNeoUtl/neoutl-branding.git`): `0ab51ecdc00569cdff0170fd58f50c3a688d09bf`（成功）
  - `i18n` (`https://codeberg.org/TeamNeoUtl/neoutl-locales.git`): `f546a55359dc18c3f6378977527a242029d3171d`（成功）
  - `sdk` (`https://codeberg.org/TeamNeoUtl/neoutl-sdk.git`): `dfe019b722577f5249981d259f12aae4884966cd`（成功）
  - `wiki` (`.gitmodules` 上の URL: `https://codeberg.org/taisho-guy/NeoUtl.wiki.git`):
    - 素の `git submodule update --init --recursive` では、Codeberg の旧オーナー名リダイレクトが Wiki リポジトリではなく本体リポジトリ `https://codeberg.org/TeamNeoUtl/NeoUtl/` に向いてしまい、`fatal: remote error: upload-pack: not our ref 143381e06262bb06985d07f61a8439596f58041c` が発生した。
    - 追跡ファイル（`.gitmodules`）を変更せず、ローカル `.git/config` の `submodule.wiki.url` を移転先の `https://codeberg.org/TeamNeoUtl/NeoUtl.wiki.git` に向けて `git submodule update --init wiki` を再実行したところ、所定の `143381e06262bb06985d07f61a8439596f58041c` で正常にチェックアウト完了した（なお `wiki` はビルドでは一切参照されない）。

---

## 2. 素のビルド（無変更状態）の結果（0-2）

本家のビルド手順（`CONTRIBUTING.md` 記載の `cargo xtask build --release`）に従い、ソースコードを 1 行も変更しない状態でビルドを実施した。

### 2.1 第1回 素のビルド（`pkgconf` 未導入状態）

- **結果**: **失敗（exit code 101）**
- **ログ保存先**: `/Users/ray_9618/.gemini/antigravity/brain/0183af41-a443-460f-914d-a1e2aa92b155/scratch/unmodified_build.log`
- **全体像**:
  1. `xtask` のビルドおよび実行は成功。
  2. `[xtask] i18n/ja.ymlからLocaleキーを生成しました` が成功。
  3. `[xtask][slang] Slang v2026.22 (slang-2026.22-macos-aarch64.zip) をダウンロード中...` → `./slang` への展開が自動完了（DXC は `if !cfg!(windows) { return; }` により自動スキップ）。
  4. ワークスペースの依存クレート・シェーダビルド（`slangc` による Slang → WGSL 変換）が進行したが、`ffmpeg-sys-next v9.0.0` の `build.rs` 実行時に `pkg-config` コマンドが見つからず停止した。
- **エラーログ原文（抜粋）**:
  ```text
  error: failed to run custom build command for `ffmpeg-sys-next v9.0.0`

  Caused by:
    process didn't exit successfully: `/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/release/build/ffmpeg-sys-next-7ae6b66f98bdab27/build-script-build` (exit status: 101)
    --- stderr

    thread 'main' (4198229) panicked at /Users/ray_9618/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ffmpeg-sys-next-9.0.0/build.rs:640:47:
    called `Result::unwrap()` on an `Err` value:
    pkg-config exited with status code 1
    > PKG_CONFIG_ALLOW_SYSTEM_LIBS=1 PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 pkg-config --libs --cflags libavutil libavformat libavfilter libavdevice libswscale libswresample libavcodec

    The system library `libavutil` required by crate `ffmpeg-sys-next` was not found.
    The file `libavutil.pc` needs to be installed and the PKG_CONFIG_PATH environment variable must contain its parent directory.
    The pkg-config command could not be found.
  ```

### 2.2 第2回 素のビルド（`brew install pkgconf` 導入後・ソースコード無変更状態）

- **結果**: **ワークスペース全クレートのビルドはエラー 0・警告 0 で完走したが、`xtask` 後処理の `maolan-plugin-host v0.0.7` インストール工程で失敗（exit code 1）**
- **ログ保存先**: `/Users/ray_9618/.gemini/antigravity/brain/0183af41-a443-460f-914d-a1e2aa92b155/scratch/unmodified_build_with_pkgconf.log`
- **全体像**:
  1. `cargo build --workspace --exclude xtask --release` 相当の本体バイナリ（`target/release/NeoUtl`）および全プラグイン（objects 9個、effects 23個、decoders 2個、easing 1個）のコンパイル・リンクが **1行のコード修正もなしに `Finished release profile [optimized + debuginfo] target(s) in 2m 20s` で成功**した。
  2. 直後に `xtask::stage_plugin_host`（`xtask/src/main.rs:239`）が `cargo install maolan-plugin-host --version 0.0.7 --locked --root target/plugin-host-install` を実行した際、外部クレート `maolan-plugin-host v0.0.7` の macOS 条件コンパイル不備により 4 個のコンパイルエラー（`E0428`, `E0433`, `E0308`×2）が発生して停止した。
- **エラーログ原文（抜粋）**:
  ```text
     CompilingNeoUtl v0.8.11 (/Users/ray_9618/Desktop/Git/NeoUtl-macOS)
      Finished `release` profile [optimized + debuginfo] target(s) in 2m 20s
  [xtask] maolan-plugin-host 0.0.7 をビルド中...
      Updating crates.io index
    Installing maolan-plugin-host v0.0.7
  ...
  error[E0428]: the name `default_clap_search_roots` is defined multiple times
     --> /Users/ray_9618/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/maolan-plugin-host-0.0.7/src/scan.rs:435:1
      |
  414 | fn default_clap_search_roots() -> Vec<PathBuf> {
      | ---------------------------------------------- previous definition of the value `default_clap_search_roots` here
  ...
  435 | fn default_clap_search_roots() -> Vec<PathBuf> {
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `default_clap_search_roots` redefined here

  error[E0433]: cannot find module or crate `paths` in this scope
    --> /Users/ray_9618/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/maolan-plugin-host-0.0.7/src/vst3/mod.rs:30:9
     |
  30 |         paths::default_vst3_search_roots()
     |         ^^^^^ use of unresolved module or unlinked crate `paths`

  error[E0308]: mismatched types
      --> /Users/ray_9618/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/maolan-plugin-host-0.0.7/src/vst3/interfaces.rs:1485:44
       |
  1485 |             name: unsafe { c_char_slice_to(&raw.name) },
       |                            --------------- ^^^^^^^^^ expected `&[i8]`, found `&[u8; 128]`

  error[E0308]: mismatched types
      --> /Users/ray_9618/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/maolan-plugin-host-0.0.7/src/vst3/interfaces.rs:1487:50
       |
  1487 |             short_name: unsafe { c_char_slice_to(&raw.shortName) },
       |                                  --------------- ^^^^^^^^^^^^^^ expected `&[i8]`, found `&[u8; 128]`

  For more information about this error, try `rustc --explain E0308`.
  error: could not compile `maolan-plugin-host` (lib) due to 4 previous errors
  Error: "cargo" "install" "maolan-plugin-host" "--version" "0.0.7" "--locked" "--root" "/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/plugin-host-install" 失敗
  ```

---

## 3. ブロッカー台帳（0-3）

### 3.1 実測で検出されたビルド・起動・テスト時のブロッカー一覧

| ID | 発生箇所（クレート名・ファイルパス・行番号） | 事象 / エラーメッセージ抜粋 | 分類 | 深刻度 | 見通し・Phase 1 での対応状況 |
|---|---|---|---|---|---|
| B-01 | `ffmpeg-sys-next v9.0.0` (`build.rs:640`) | `The pkg-config command could not be found.` | ビルド設定 / 依存ライブラリ | 致命的（ビルド不能） | **解決済み**: Homebrew で `pkgconf 3.0.10` を導入することで解消。コード変更不要。 |
| B-02 | `xtask/src/main.rs:205` (`const PLUGIN_HOST_VERSION: &str = "0.0.7";`) | `maolan-plugin-host v0.0.7` の `cargo install` 時に `E0428` (`src/scan.rs:435`), `E0433` (`src/vst3/mod.rs:30`), `E0308` (`src/vst3/interfaces.rs:1485, 1487`) でコンパイル失敗 | 依存ライブラリ / ビルド設定 | 致命的（`xtask build` 完走不能） | **最小修正で解決済み**: `crates/host-adapter/Cargo.toml` では既に macOS 修正済みの `maolan-plugin-host = "0.0.8"` が使われていたが、`xtask/src/main.rs:205` のみ `"0.0.7"` のまま取り残されていた。定数を `"0.0.8"` に更新（1行変更）し完全解消。 |
| B-03 | `src/infra/gpu_shared.rs:240-260` (`init_shared_gpu`) | 起動直後に `Error: RequestDeviceError { inner: Core(UnsupportedFeature(Features { features_wgpu: FeaturesWGPU(TEXTURE_FORMAT_NV12), features_webgpu: FeaturesWebGPU(0x0) })) }` で即時終了。さらに `limits.max_storage_buffers_per_shader_stage = 1` 固定および `neo_media_ffmpeg::set_shared_wgpu_device` 未呼び出し | GPU・描画 | 致命的（起動不能） | **最小修正で解決済み**: `#[cfg(target_os = "macos")]` にて ① `adapter.features().contains(wgpu::Features::TEXTURE_FORMAT_NV12)` でサポート時のみ有効化、② `wgpu::Limits::default()` を使用（`neo-media-ffmpeg` の色変換 Compute Shader が storage buffer を2個使用するため）、③ `neo_media_ffmpeg::set_shared_wgpu_device(device.clone(), queue.clone())` を呼ぶよう修正。 |
| B-04 | `.gitmodules:10-12` (`wiki` submodule URL `https://codeberg.org/taisho-guy/NeoUtl.wiki.git`) | `fatal: remote error: upload-pack: not our ref 143381e06262bb06985d07f61a8439596f58041c` | その他（Git Submodule） | 軽微（回避策あり・ビルド非影響） | **回避済み**: Codeberg のリポジトリ移転に伴うリダイレクト不備。ローカル `.git/config` の URL を `https://codeberg.org/TeamNeoUtl/NeoUtl.wiki.git` に設定してチェックアウト完了。 |
| B-05 | `src/ecs/systems/mod.rs:441, 448` (`#[cfg(test)] mod tests`) | `cargo test --bin NeoUtl` 実行時に `error[E0308]: mismatched types: expected StrRef, found &str` | その他（上流既存テストの不整合） | 軽微（通常ビルド・実行には非影響） | **原因特定・切り分け済み**: 本家の SDK 変更（`ObjectMeta.stable_id` が `&str` から `StrRef` に変更）に `src/ecs/systems/mod.rs` のテストコードが追随していなかった本家既存の不整合（macOS 固有ではない）。該当2箇所を `StrRef::from_str(...)` にすれば 53 件全通過することも実測確認済み（詳細は §6.1）。 |

### 3.2 必須調査7項目の実物コード確認と判定（0-3 必須項目）

エラーの有無にかかわらず、実物コードを読んで確認した7項目の判定結果は以下の通り。

#### 項目1: ルート `Cargo.toml` の `eframe` 指定（`default-features = false, features = ["wgpu", "default_fonts", "x11", "wayland"]`）が macOS ビルドで成立するか
- **確認箇所**: `Cargo.toml:26`、および Cargo キャッシュ内の `eframe-0.36.2/Cargo.toml`、`egui-winit-0.36.2/Cargo.toml`、`winit-0.30.13/Cargo.toml`、`winit-0.30.13/build.rs`
- **分類**: ビルド設定 / 依存ライブラリ
- **深刻度**: 問題なし（修正不要）
- **実物確認による判定**: **無変更のまま macOS で完全に成立する。**
  - `eframe` の `x11` / `wayland` feature は `egui-winit` および `winit` の同名 feature に転送されるが、`egui-winit` と `winit` の `Cargo.toml` では `wayland-client` / `x11-dl` / `smithay-clipboard` 等の Linux 系クレート依存がすべて `[target.'cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))'.dependencies]` のターゲット条件付き依存として宣言されている。
  - また `winit-0.30.13/build.rs` でも `x11_platform` / `wayland_platform` は `all(feature = "...", not(any(target_os = "macos", windows, ...)))` として定義されており、macOS ターゲットでは `objc2-app-kit` 等の AppKit/Cocoa バックエンドのみがコンパイルされる。

#### 項目2: `cfg(target_os = "linux")` = vulkan / `cfg(target_os = "windows")` = dx12 という `wgpu-hal` の指定に対し、macOS（Metal）側の指定が要るか
- **確認箇所**: `Cargo.toml:94-98`、`src/infra/gpu_shared.rs:218-283`、`crates/media/ffmpeg/src/colorconv.rs:44-80`
- **分類**: GPU・描画
- **深刻度**: `Cargo.toml` は修正不要だが、`src/infra/gpu_shared.rs` の Metal 初期化コードに **致命的（起動不能）バグ（B-03）** があり最小修正が必要だった。
- **実物確認による判定**:
  1. **`Cargo.toml` の `wgpu-hal` 指定は不要**: NeoUtl が `wgpu-hal` を直接呼んでいるのは `src/infra/gpu_shared.rs:100-215` の `#[cfg(target_os = "linux")]` ブロック（Vulkan の `VK_EXT_queue_family_foreign` 等を有効化して `from_hal` する処理）のみである。`wgpu 30.0.1` 本体は Apple ターゲット向けにデフォルトで `wgpu-core-deps-apple` を介して Metal バックエンドを有効化しており、`src/infra/gpu_shared.rs:221` でも既に `#[cfg(target_os = "macos")] let backends = wgpu::Backends::METAL;` が実装されていた。
  2. **`src/infra/gpu_shared.rs` の macOS デバイス初期化に3点のバグ（B-03）**:
     - `src/infra/gpu_shared.rs:243` で `#[cfg(target_os = "macos")] let required_features = wgpu::Features::TEXTURE_FORMAT_NV12;` を無条件要求していたが、`wgpu 30` の Metal バックエンドで `TEXTURE_FORMAT_NV12` がサポートされない環境（Apple M1 実機）では `request_device` が `UnsupportedFeature(TEXTURE_FORMAT_NV12)` で即時失敗し起動不能となる。
     - `src/infra/gpu_shared.rs:241` で `limits.max_storage_buffers_per_shader_stage = 1;` に制限していたが、`crates/media/ffmpeg/src/colorconv.rs:44-80` の色変換 Compute Shader は `@binding(0) var<storage, read>` と `@binding(1) var<storage, read_write>` の **2個の storage buffer** を要求するため、上限 `1` のままでは動画デコード時にパイプライン生成失敗となる。
     - Linux ブロック（203行目）にはある `neo_media_ffmpeg::set_shared_wgpu_device(device.clone(), queue.clone());` が非 Linux ブロック（218〜270行目）で抜けており、共有 wgpu デバイスと `NeoMediaCache` が `neo-media-ffmpeg` に渡されていなかった。
  3. **対応**: `src/infra/gpu_shared.rs` の `#[cfg(target_os = "macos")]` 分岐でこの3点を最小修正し、Metal バックエンドでの正常起動およびレンダーパイプライン単体テスト全件パスを確認した。

#### 項目3: objects / effects プラグインの動的ロード: 拡張子（`.so` / `.dylib`）と探索パスの扱い、署名・Library Validation の影響
- **確認箇所**:
  - `xtask/src/main.rs:81-89` (`cdylib_filename`)
  - `src/objects/loader.rs:253`, `src/objects/mod.rs:34-47` (`default_objects_dir`)
  - `src/effects/loader.rs:360`, `src/effects/mod.rs:146-159` (`default_effects_dir`)
  - `crates/media/runtime/src/loader.rs:169`, `src/media/mod.rs:16-30` (`default_decoders_dir`)
  - `src/easings/loader.rs:154`, `src/easings/mod.rs:14-27` (`default_easings_dir`)
  - `src/infra/hot_reload.rs:87` (`is_plugin_Binary`)
- **分類**: プラグイン読み込み
- **深刻度**: 問題なし（Phase 1 修正不要）
- **実物確認による判定**: **本家コードに既に macOS (`.dylib` および `.app/Contents/Resources` フォールバック) 対応が実装済みであり、実機起動で全プラグインのロード・登録成功を確認した。**
  - `xtask/src/main.rs:83` は `if cfg!(target_os = "macos") { format!("lib{base}.dylib") }` を返し、`target/release/{objects,effects,decoders,easings}/lib*.dylib` に配置する。
  - 各ローダー（objects, effects, decoders, easings）および `hot_reload.rs` はすべて `matches!(path.extension().and_then(|s| s.to_str()), Some("so" | "dylib" | "dll"))` で `.dylib` を対象にしており、探索パスも実行ファイル隣接の `{objects,effects,decoders,easings}` が存在しない場合は `#[cfg(target_os = "macos")]` で `../Resources/{objects,effects,decoders,easings}`（将来の `.app` バンドル構造）を参照する実装になっている。
  - ローカルビルドでは本体・各 `.dylib` とも Apple clang リンカが付与する ad-hoc 署名（Linker-signed）で統一されており、Hardened Runtime の Library Validation によるブロックも発生しない（起動ログにて objects 9個、effects 23個、decoders 3種、easing 1個の全登録を実測確認）。将来 Phase 4/5 で Developer ID 署名・公証を行う際は、本体と全同梱 `.dylib` / `maolan-plugin-host` を同一 Team ID で署名する必要がある。

#### 項目4: `build.rs` / `xtask` の Slang コンパイル経路（`xtask` 側の DXC 処理含む）: 使う外部ツール（`slangc` 等）の入手方法と macOS arm64 対応
- **確認箇所**:
  - `xtask/src/slang.rs:1-85`
  - `xtask/src/dxc.rs:139-143`
  - `crates/effect/shader-build/src/lib.rs:59-105`
  - `crates/object/shader-build/src/lib.rs:58-92`
  - `crates/media/ffmpeg/build.rs:1-130`
- **分類**: シェーダ / ビルド設定
- **深刻度**: 問題なし（修正不要）
- **実物確認による判定**: **本家コードが macOS arm64 (`macos-aarch64`) に完全対応している。**
  - `xtask/src/slang.rs:62-84` の `detect_platform_asset_keyword()` に `("macos", "aarch64") => "macos-aarch64"` および `("macos", "x86_64") => "macos-x86_64"` が定義されており、`cargo xtask build` 実行時に GitHub (`shader-slang/slang`) から `slang-2026.22-macos-aarch64.zip` を `./slang` に自動ダウンロード・展開し、`chmod 0o755` まで行って `PATH` 先頭に追加する。
  - `xtask/src/dxc.rs:140` の `ensure_dxc()` は先頭で `if !cfg!(windows) { return; }` となっており、macOS では DXC ダウンロード・配置を一切行わない。
  - 各クレートの `build.rs` は `slangc` を用いて `.slang` シェーダを WGSL（`-target wgsl`）にコンパイルしており、実行時には `wgpu` / `naga` が WGSL から Metal Shading Language (MSL) へ変換する。

#### 項目5: FFmpeg へのリンク方式（静的／動的、`pkg-config` 経由か）と、将来 `.app` に同梱する場合の `dylib` 問題の有無
- **確認箇所**: `crates/media/ffmpeg/Cargo.toml`、`target/release/NeoUtl` の `otool -L` 実測出力
- **分類**: 依存ライブラリ / ファイル・パス
- **深刻度**: Phase 1（ローカル起動）では軽微（`pkgconf` 導入のみで解決）、**Phase 4（`.app` バンドル配布）では重大課題**
- **実物確認による判定**:
  - `ffmpeg-next v9.0.0` / `ffmpeg-sys-next v9.0.0` が `pkg-config`（Homebrew の `pkgconf`）経由で Homebrew の FFmpeg 9.0.2 を**動的リンク**している。
  - `otool -L target/release/NeoUtl` の実測結果（抜粋）:
    ```text
    /opt/homebrew/opt/ffmpeg/lib/libavutil.61.dylib (compatibility version 61.0.0, current version 61.1.102)
    /opt/homebrew/opt/ffmpeg/lib/libavformat.63.dylib (compatibility version 63.0.0, current version 63.1.102)
    /opt/homebrew/opt/ffmpeg/lib/libavfilter.63.dylib -> libavfilter.12.dylib (current version 12.1.102)
    /opt/homebrew/opt/ffmpeg/lib/libavdevice.63.dylib (compatibility version 63.0.0, current version 63.1.102)
    /opt/homebrew/opt/ffmpeg/lib/libswscale.10.dylib (compatibility version 10.0.0, current version 10.1.102)
    /opt/homebrew/opt/ffmpeg/lib/libswresample.7.dylib (compatibility version 7.0.0, current version 7.1.102)
    /opt/homebrew/opt/ffmpeg/lib/libavcodec.63.dylib (compatibility version 63.0.0, current version 63.1.102)
    ```
  - **将来の `.app` 同梱時の課題**: バイナリ内の参照パスが `/opt/homebrew/opt/ffmpeg/lib/*.dylib` の絶対パスになっているため、他環境へ配布する `.app` バンドルを作る場合（Phase 4）には、`Contents/Frameworks/` 等へ FFmpeg（およびその依存 dylib 群）を同梱して `install_name_tool` / `@rpath` で書き換えるか、スタティックリンク構成の FFmpeg を自前ビルドしてリンクする必要がある。

#### 項目6: 日本語まわり: `egui-system-fonts` で日本語フォントが拾えるか、`winit` / `egui` 経由の日本語 IME 入力が成立するか
- **確認箇所**: `src/app/setup.rs`、`egui-system-fonts 0.36.0`、`font-kit 0.14.3`、および Phase 1 実機起動画面
- **分類**: フォント・IME
- **深刻度**: 日本語表示は問題なし、日本語 IME 変換確定の細部挙動は Phase 2 で継続確認
- **実物確認による判定**:
  - **日本語フォント表示（実測確認済み・問題なし）**: `egui-system-fonts 0.36.0` および `font-kit 0.14.3` は macOS で `CoreText.framework` を使用してシステムフォント（ヒラギノ角ゴシック等）を自動取得する。Phase 1 の実機起動およびスクリーンショット（`target/macos_smoke_test/neoutl_macos_phase1.png`）にて、「新規プロジェクト」「映像」「音声」「作成して開く」「既存プロジェクト」「並べ替え: 名前 ↑」等の日本語 UI が文字化け（豆腐）一切なく鮮明に描画されていることを目視確認した。
  - **日本語 IME 入力**: `eframe 0.36.2` / `winit 0.30.13` の AppKit バックエンドで IME イベント処理自体は組み込まれているが、プロジェクト作成後のテキストオブジェクト編集や入力欄での日本語かな漢字変換・インラインプリエディット表示・確定操作の実機 E2E 検証は Phase 2 の対象となるため、現時点では**「未確認（Phase 2 で実機検証）」**とする。

#### 項目7: 自動更新（`self-replace`）の macOS での扱い（このフェーズでは無効化の要否だけ判定し、実装はしない）
- **確認箇所**: `src/app/update.rs:1-210`（特に `find_binary_in_dir` の `197-205` 行目）
- **分類**: その他（自動更新）
- **深刻度**: 軽微（起動・通常動作のブロッカーではないが、Phase 4 で無効化が必要）
- **実物確認による判定**:
  - `src/app/update.rs` はユーザーが UI 上で更新チェック／更新適用を実行したときにのみ動作し、起動時に勝手にバイナリを書き換えることはないため、**Phase 1 では無効化実装不要**。
  - ただし以下の3点の理由により、**Phase 4（`.app` バンドル化・配布準備）で macOS 向けに無効化（または Homebrew / DMG 案内へ差し替え）が必要**と判定する:
    1. `src/app/update.rs:11` の `REPO_OWNER = "taisho-guy"`, `REPO_NAME = "NeoUtl"` は本家 Codeberg リポジトリを指しており、本家は現在 macOS ビルド成果物をリリースしていない。
    2. `src/app/update.rs:197-205` には `if cfg!(target_os = "macos") { ... dir.join("NeoUtl.app/Contents/MacOS/NeoUtl") }` というコードがあるものの、`self_replace::self_replace(&extracted_bin)` は単一実行ファイルのみを置換し、同梱プラグイン（`.dylib`）やシェーダ・リソースを更新しない。
    3. macOS の `.app` バンドル内で `Contents/MacOS/NeoUtl` バイナリだけを後から書き換えると、`.app` バンドル全体のコード署名（Code Signature）が破損し、次回起動時に Gatekeeper / macOS セキュリティ機構により起動が拒否される。

---

## 4. ライセンス一次棚卸しの結果（0-4）

### 4.1 本体および Submodule のライセンス確認結果（実物確認）

| 対象 | パス / バージョン | ライセンス実測結果 | 根拠ファイル・備考 |
|---|---|---|---|
| **NeoUtl 本体** | ルート (`v0.8.11`, `0ead3935`) | **`AGPL-3.0-or-later`** | `Cargo.toml:6` (`license = "AGPL-3.0-or-later"`), ルート `LICENSE`（GNU AGPL v3 全文） |
| **`assets` submodule** (`neoutl-branding`) | `assets/` (`0ab51ecd`) | **`CC0 1.0 Universal`（パブリックドメイン献呈 / 商標権留保条項あり）** | `assets/LICENSE`（CC0 1.0 全文）および `assets/README.md` に「本リポジトリ内のロゴ・アイコン等のアセットは CC0 1.0 Universal の下で提供されており、商用・非商用を問わず、改変や再配布も含めて自由に利用できます」と明記。ただし CC0 第4条a項に基づき「商標権およびブランド保護の観点から、公式と誤認される形での利用は認められません」との注意書きあり。自己利用は問題なく、将来第三者配布する場合は「非公式 macOS ビルド」等の明記または差し替えが望ましい。 |
| **`i18n` submodule** (`neoutl-locales`) | `i18n/` (`f546a553`) | **`CC0 1.0 Universal`** | `i18n/README.md` に `CC0 1.0 Universal` と明記 |
| **`sdk` submodule** (`neoutl-sdk`) | `sdk/` (`dfe019b7`) | **`MIT OR Apache-2.0`**（公開 SDK API クレート群） | `sdk/neoutl/0.8.11/rust/Cargo.toml` 等で `license = "MIT OR Apache-2.0"`（一部内部クレートは workspace 継承の `AGPL-3.0-or-later`） |

### 4.2 外部ネイティブ依存・ツール・主要 Rust クレートのライセンス一覧

| 依存コンポーネント | バージョン / 導入形態 | ライセンス | 判定・備考 |
|---|---|---|---|
| **FFmpeg** (Homebrew) | `9.0.2`（動的リンク） | **`GPL-3.0-or-later` (GPLv3+)** | §1.2 の configure フラグに **`--enable-gpl --enable-version3`** および GPL ライブラリ（`--enable-libx264`, `--enable-libx265`）が含まれているため、LGPL ではなく **GPLv3+** ビルドと判定できる。NeoUtl 本体（`AGPL-3.0-or-later`）とは GNU AGPLv3 第13条（GPLv3 との結合許可条項）により互換性がある（配布時は全体として AGPLv3+ の条件および対応ソース提供義務が適用される）。なお `--enable-openssl` でリンクされている OpenSSL 3.6.4 は `Apache-2.0` ライセンスであり、GPLv3 / AGPLv3 と互換である。 |
| **LuaJIT** (`mlua` / `luajit-src`) | `mlua 0.12.2` / `luajit-src 210.7.4+resty92c1790`（`vendored` 静的リンク） | **`MIT`** | `mlua` (`MIT`) および LuaJIT (`MIT`) ともに MIT ライセンス。 |
| **Slang** (`slangc`) | `v2026.22` (`slang-2026.22-macos-aarch64.zip`) | **`Apache-2.0 WITH LLVM-exception`** | ビルド時に `.slang` を `.wgsl` に変換する外部コンパイラとして使用。生成された WGSL 文字列のみがバイナリに埋め込まれ、Slang ランタイム自体はリンクされない（`xtask` が `target/release/slang/` にコピーしているため、配布時は同梱要否を整理すること）。 |
| **`egui` / `eframe` / `egui-wgpu` / `egui-winit`** | `0.36.2` | **`MIT OR Apache-2.0`** | 許容型（Permissive）ライセンス。 |
| **`wgpu` / `wgpu-core` / `wgpu-hal` / `naga`** | `30.0.1` | **`MIT OR Apache-2.0`** | 許容型（Permissive）ライセンス。 |
| **`ffmpeg-next` / `ffmpeg-sys-next`** | `9.0.0` | **`MIT OR Apache-2.0`**（ラッパークレート自体） | リンク先の FFmpeg 本体のライセンス（上記 GPLv3+）が支配的となる。 |
| **`shipyard`** | `0.11.5` | **`MIT OR Apache-2.0`** | ECS クレート。 |
| **`rodio` / `cpal` / `symphonia`** | `0.22.2` / `0.17.3` / `0.5.5` | **`MIT OR Apache-2.0`** (`symphonia` は `MPL-2.0`) | `symphonia` (`MPL-2.0`) は AGPLv3 と互換。 |
| **`maolan-host-adapter` / `maolan-plugin-host`** | `0.1.0` / `0.0.8` | **`BSD-2-Clause`** | 音声プラグイン（VST3 / CLAP / LV2）ホスト。 |
| **`font-kit` / `egui-system-fonts`** | `0.14.3` / `0.36.0` | **`MIT OR Apache-2.0`** | フォント探索・読み込み。 |
| **`rfd` / `notify` / `tokio` / `serde`** | `0.17.2` / `8.2.0` / `1.53.2` / `1.0.228` | **`MIT OR Apache-2.0`** (`notify` は `CC0-1.0`) | いずれも AGPLv3 と互換。 |

---

## 5. 変更ファイルの一覧と各変更の目的（1-1）

### 5.1 `git status -s` および `git diff --stat` の実測出力

```text
$ git status -s
 M src/infra/gpu_shared.rs
 M xtask/src/main.rs
?? MACOS_PHASE0_1_REPORT.md

$ git diff --stat
 src/infra/gpu_shared.rs | 28 +++++++++++++++++++++++-----
 xtask/src/main.rs       |  2 +-
 2 files changed, 24 insertions(+), 6 deletions(-)
```

### 5.2 各変更ファイルの目的（1行説明）

1. **`xtask/src/main.rs`**: `PLUGIN_HOST_VERSION` を `"0.0.7"`（macOS ビルド不可）から、`crates/host-adapter/Cargo.toml` と同一の `"0.0.8"`（macOS コンパイルエラー修正済み）へ更新。
2. **`src/infra/gpu_shared.rs`**: `#[cfg(target_os = "macos")]` にて、`TEXTURE_FORMAT_NV12` をアダプタ対応時のみ要求するよう変更し、`wgpu::Limits::default()` の使用および `neo_media_ffmpeg::set_shared_wgpu_device` の呼び出しを追加（Linux / Windows の挙動は完全維持）。

---

## 6. テスト結果（1-2, 1-3）

### 6.1 既存テスト（`cargo test`）の結果と内訳

1. **無変更のテストコードに対する `cargo test`（`cargo test --bin NeoUtl`）**:
   - **結果**: `src/ecs/systems/mod.rs` の 441 行目・448 行目（`#[cfg(test)] mod tests` 内）にて、本家既存の型不一致エラー（`E0308: expected StrRef, found &str`）によりコンパイル失敗。
   - **切り分け**: 本家で `neoutl_object_api::ObjectMeta.stable_id` の型が `&'static str` から `StrRef` に変更された際、`src/ecs/systems/mod.rs` のテスト用ダミー定数 `DUMMY_CAMERA_META` / `DUMMY_LIGHT_META` が更新漏れになっていたものであり、macOS 対応修正に起因するものではない。
2. **サブクレート群のテスト（`cargo test --workspace --exclude NeoUtl`）**:
   - **結果**: **全 55 テストターゲットがコンパイル・実行成功（17 passed, 0 failed, 7 doctests ignored）**
   - 内訳:
     - `neo-media-swscale` (`tests/scale_accuracy.rs`): **3 passed, 0 failed**
     - `neoutl-color`: **4 passed, 0 failed**
     - `neoutl-effect-api`: **1 passed, 0 failed**
     - `neoutl-effect-displacement-map-translate`: **3 passed, 0 failed**
     - `neoutl-expression-api`: **2 passed, 0 failed**
     - `neoutl-media-runtime`: **1 passed, 0 failed**
     - `neoutl-object-api`: **1 passed, 0 failed**
     - `neoutl-sdk`: **1 passed, 0 failed** (doctests: 7 ignored)
     - `neoutl-shared-abi`: **1 passed, 0 failed**
     - その他 46 クレート: 0 tests (0 failed)
3. **（参考実測）`src/ecs/systems/mod.rs:441, 448` の2箇所を `StrRef::from_str(...)` に一時修正した場合の `cargo test --bin NeoUtl`**:
   - **結果**: **53 passed, 0 failed, 0 ignored**（実行時間 1.16s）
   - Metal 実機 GPU を用いる `renderer::pipeline::tests` の全 6 テスト（`render_engine_new_succeeds`, `render_empty_scene_clears_target`, `distinct_effect_chains_render_independently`, `effect_chain_does_not_leak_to_adjacent_object`, `resize_render_target_updates_dimensions_and_survives_render`, `test_reconfigure_accelerator_rebinds_and_renders`）および ECS / Project / Protobuf スキーマの全 47 テストが **macOS 上で 100% パス（合計 70 passed, 0 failed）** することを確認した（確認後、非 macOS 共通コード保護のため `src/ecs/systems/mod.rs` は無変更状態に戻してある）。

### 6.2 起動スモークテストの手順と実測結果

- **スモークテストスクリプト配置先（再実行可能）**:
  - リポジトリ内（`.gitignore` 対象の `target/` 配下）: `/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/macos_smoke_test/macos_smoke_test.sh`
  - 作業用スクラッチ領域: `/Users/ray_9618/.gemini/antigravity/brain/0183af41-a443-460f-914d-a1e2aa92b155/scratch/macos_smoke_test.sh`
- **実行手順**:
  ```bash
  /Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/macos_smoke_test/macos_smoke_test.sh
  ```
  1. `./target/release/NeoUtl` をバックグラウンド起動し、標準出力・標準エラー出力をログに記録。
  2. 8秒待機してプロセス生存を確認し、ウィンドウを最前面化して `screencapture -x` で画面キャプチャを取得。
  3. さらに24秒待機（起動から合計 **32秒間** >= 30秒）し、`kill -0` でプロセスがパニック・クラッシュなく生存し続けていることを検証。
  4. AppleScript 経由で標準終了操作（`Cmd+Q`）を送信し、プロセスがコード 0 で正常終了することを検証。
- **スモークテスト実行出力（実測）**:
  ```text
  [INFO] Starting NeoUtl (./target/release/NeoUtl)...
  [INFO] NeoUtl PID: 97559
  [INFO] Capturing screenshot to /Users/ray_9618/.gemini/antigravity/brain/0183af41-a443-460f-914d-a1e2aa92b155/scratch/neoutl_macos_phase1.png ...
  [INFO] Waiting up to 32s total to verify 30s stability...
  [PASS] NeoUtl survived >= 30 seconds (PID 97559 alive at 32s).
  [INFO] Requesting graceful quit...
  [PASS] NeoUtl exited cleanly via Cmd+Q.
  ```
- **起動時アプリケーションログ全文（`target/macos_smoke_test/neoutl_smoke_run.log`）**:
  ```text
  [neo-media-cache][診断] 初期VRAM予算取得失敗 フォールバック容量=6適用中
  [neo-media-cache][診断] 初期RAM予算取得失敗 フォールバック容量=64適用中
  [NeoUtl] プラグイン登録: neoutl.object.audio (Audio, kind_id=0)
  [NeoUtl] プラグイン登録: neoutl.object.camera (Camera, kind_id=1)
  [NeoUtl] プラグイン登録: neoutl.object.group_control (Group Control, kind_id=2)
  [NeoUtl] プラグイン登録: neoutl.object.image (Image, kind_id=3)
  [NeoUtl] プラグイン登録: neoutl.object.light (Light, kind_id=4)
  [NeoUtl] プラグイン登録: neoutl.object.scene (Scene, kind_id=5)
  [NeoUtl] プラグイン登録: neoutl.object.shape (Shape, kind_id=6)
  [NeoUtl] プラグイン登録: neoutl.object.text (Text, kind_id=7)
  [NeoUtl] プラグイン登録: neoutl.object.video (Video, kind_id=8)
  [NeoUtl] エフェクト登録: border_blur (BorderBlur) [native]
  [NeoUtl] エフェクト登録: chromatic_aberration (ChromaticAberration) [native]
  [NeoUtl] エフェクト登録: clipping (Clipping) [native]
  [NeoUtl] エフェクト登録: color_correction (ColorCorrection) [native]
  [NeoUtl] エフェクト登録: diagonal_clipping (DiagonalClipping) [native]
  [NeoUtl] エフェクト登録: diffuse_light (DiffuseLight) [native]
  [NeoUtl] エフェクト登録: directional_blur (DirectionalBlur) [native]
  [NeoUtl] エフェクト登録: displacement_map_image (DisplacementMap(Image)) [native]
  [NeoUtl] エフェクト登録: displacement_map_rotate (DisplacementMap(Rotate)) [native]
  [NeoUtl] エフェクト登録: displacement_map_scale (DisplacementMap(Scale)) [native]
  [NeoUtl] エフェクト登録: displacement_map_scene (DisplacementMap(Scene)) [native]
  [NeoUtl] エフェクト登録: displacement_map_translate (DisplacementMap(Translate)) [native]
  [NeoUtl] エフェクト登録: drop_shadow (DropShadow) [native]
  [NeoUtl] エフェクト登録: image_loop (ImageLoop) [native]
  [NeoUtl] エフェクト登録: lens_blur (LensBlur) [native]
  [NeoUtl] エフェクト登録: mask_shape (MaskShape) [native]
  [NeoUtl] エフェクト登録: mosaic (Mosaic) [native]
  [NeoUtl] エフェクト登録: motion_blur (MotionBlur) [native]
  [NeoUtl] エフェクト登録: pixel_sorter (PixelSorter) [native]
  [NeoUtl] エフェクト登録: radial_blur (RadialBlur) [native]
  [NeoUtl] エフェクト登録: text_outline (TextOutline) [native]
  [NeoUtl] エフェクト登録: transform (Transform) [native]
  [NeoUtl] エフェクト登録: vibration (Vibration) [native]
  [NeoUtl] デコーダ登録: neoutl.media.ffmpeg (FFmpeg Video Decoder)
  [NeoUtl] デコーダ登録: neoutl.media.image (Static Image Decoder)
  [NeoUtl] デコーダ登録: neoutl.media.symphonia (Symphonia Audio Decoder)
  [NeoUtl] イージングエンジン登録: neoutl-easing-standard (Standard Easing Engine)
  [maolan-host-adapter] プラグイン走査開始 format=vst3 path=--system
  [maolan-host-adapter] プラグイン走査完了 format=vst3
  [maolan-host-adapter] プラグイン走査開始 format=clap path=--system
  [maolan-host-adapter] プラグイン走査完了 format=clap
  [maolan-host-adapter] プラグイン走査開始 format=lv2 path=--system
  [maolan-host-adapter] プラグイン走査完了 format=lv2
  ```

### 6.3 スクリーンショットのファイルパスと目視確認結果

- **スクリーンショット保存先**:
  - `/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/macos_smoke_test/neoutl_macos_phase1.png`
  - `/Users/ray_9618/.gemini/antigravity/brain/0183af41-a443-460f-914d-a1e2aa92b155/scratch/neoutl_macos_phase1.png`
- **目視確認結果**:
  1. macOS のメニューバー左上に `NeoUtl` が表示され、画面中央に `NeoUtl` メインウィンドウ（信号機ボタン付きタイトルバー `NeoUtl`）が正常に描画されている。
  2. 初期画面（プロジェクトランチャー）の「新規プロジェクト」セクション（映像: `30 fps`, `1920 px`, `1080 px` / 音声: `2 ch`, `48000 Hz` / `名前` 入力フィールド / 青色の `作成して開く` ボタン）および「既存プロジェクト」セクション（`検索` フィールド / `並べ替え: 名前 ↑` / `選択モードに入る` / `コピー` / `削除` / `プロジェクトがありません`）が崩れ・重なり・描画ノイズ一切なく表示されている。
  3. 日本語フォントがシステムフォント（CoreText）経由で鮮明にレンダリングされており、文字化け・文字切れは皆無である。

![NeoUtl macOS Phase 1 Launch Screenshot](target/macos_smoke_test/neoutl_macos_phase1.png)

---

## 7. スタブ化・無効化した機能の一覧（1-1）

- **なし**
  - 本体・オブジェクトプラグイン（9種）・エフェクトプラグイン（23種）・メディアデコーダ（FFmpeg / Image / Symphonia の3種）・標準イージングエンジン・Lua スクリプト群・音声プラグインホスト（`maolan-plugin-host` による VST3 / CLAP / LV2 スキャン）のすべてが、スタブ化や機能無効化を一切行うことなくネイティブにビルド・起動・登録完了した。

---

## 8. 後続フェーズ（Phase 2〜5）への申し送り事項

1. **Phase 2（基本編集の E2E 動作確認）への申し送り**:
   - **VideoToolbox ハードウェアデコードとソフトウェアデコードの検証**: `crates/media/ffmpeg/src/hwaccel.rs` には `#[cfg(target_os = "macos")]` で `AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX` が優先設定されている。ただし VideoToolbox デコードフレームから wgpu Metal テクスチャへのゼロコピー経路は未実装であり、CPU リードバック + Compute Shader（`colorconv.rs`）による色変換経路が走る設計になっている。実動画読み込み・プレビュー再生・シーク・書き出し（エンコード）の E2E 動作とフレームレートを検証すること。
   - **日本語 IME 入力・テキストオブジェクト描画の検証**: ランチャーのプロジェクト名入力欄や Text オブジェクトでの日本語 IME 変換・確定操作、およびフォント選択 UI の動作を実機確認すること。
   - **音声出力（`rodio` / CoreAudio）の検証**: タイムライン再生時の音声出力および Audio オブジェクトの波形表示・シーク追従を確認すること。
   - **本家テスト `src/ecs/systems/mod.rs:441, 448` の型修正**: 必要に応じて `StrRef::from_str(...)` の2行修正を適用または上流へフィードバックすること。
2. **Phase 3（プラグイン・シェーダ・スクリプト本対応）への申し送り**:
   - 全 9 オブジェクト・23 エフェクト・Lua スクリプト（`target/release/scripts/`）は既にロード・登録まで完了しているため、Phase 3 では各エフェクト・カメラ・ライト・Lua スクリプトを実際にタイムライン上で適用した際の WGSL→MSL シェーダ実行結果を重点検証すること。
3. **Phase 4（`.app` バンドル化・パッケージング）への申し送り**:
   - **FFmpeg 動的ライブラリのバンドル化**: `target/release/NeoUtl` が `/opt/homebrew/opt/ffmpeg/lib/libav*.dylib` を絶対パス参照しているため、単体で動く `.app` にするには `Contents/Frameworks/` への dylib 同梱と `install_name_tool` による `@rpath` / `@executable_path/../Frameworks/` 書き換えが必要。
   - **プラグイン・アセット配置**: ローダー側は既に `../Resources/{objects,effects,decoders,easings}` フォールバックを持っているが、`scripts/`（Lua）や `maolan-plugin-host` バイナリの探索パスが `.app` バンドル実行時（カレントディレクトリが `/` 等になる場合）にも解決されるか確認・調整が必要。
   - **自動更新（`src/app/update.rs`）の無効化**: `.app` のコード署名破損および本家 Linux/Windows リリースとの混線を防ぐため、macOS ビルドでは自動更新機能を無効化すること。
