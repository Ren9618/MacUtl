# macOS Phase 5 報告書 — MacUtl 公開・初回リリース

- **日付**: 2026-11-12
- **対象**: MacUtl 0.8.11（macOS fork、ベース: NeoUtl v0.8.11 / `0ead39350db55996810de53da79d02fbc98355f5`）
- **リポジトリ**: <https://github.com/Ren9618/MacUtl>（public）
- **リリース**: <https://github.com/Ren9618/MacUtl/releases/tag/v0.8.11-mac.1>
- **タグ**: `v0.8.11-mac.1`（annotated tag `3e3cdc0148dd9c1d6c72a46c620fb5f2f39c01f9` → `eae56b199c24011cfd06a9ce3683a1dc50b34052`）
- **結論**: ライセンスゲートは **PASS**。macOS 向けネットワーク送信はコードレベルで無効化し、MacUtl へのリネーム、回帰試験、ad-hoc 署名パッケージ作成、GitHub 公開、リリース済み成果物の再ダウンロード検証まで完了した。公開ブロッカーはない。

---

## 1. 要約

| 項目 | 結果 |
|---|---|
| ライセンス監査 | **PASS**（AGPL 非互換ライセンスなし、FFmpeg は GPL-3.0-or-later で GPLv3 §13 により両立） |
| ネットワーク監査 | 上流の更新 API と GlitchTip クラッシュレポートの 2 系統のみ。macOS ビルドでは両方とも無効 |
| リネーム | ユーザー可視名を **MacUtl** に変更。内部クレート名・`.npb`・プラグイン ABI は維持 |
| 回帰試験 | `cargo fmt` 合格、`cargo test --workspace --exclude NeoUtl` は 80 件の `test result: ok`、failed 0 |
| 720p 書き出し | 動画＋画像プロジェクト 391 フレーム完走（1280×720、13.000 秒、クラッシュ 0） |
| リリース成果物 | DMG 40,814,500 bytes / ZIP 36,332,190 bytes。GitHub の asset digest とローカル SHA-256 が一致 |
| 再ダウンロード検証 | DMG をマウントして起動、ZIP を展開して署名検証、720p 書き出しが参照ハッシュと一致 |
| 署名 | ad-hoc のみ（`codesign --sign -`）。Apple Developer ID / notarization は未使用 |
| 対応環境 | Apple Silicon（arm64）thin / macOS 27.0 以降 |

---

## 2. 公開情報

### 2.1 URL

| 用途 | URL |
|---|---|
| リポジトリ | <https://github.com/Ren9618/MacUtl> |
| リリース | <https://github.com/Ren9618/MacUtl/releases/tag/v0.8.11-mac.1> |
| DMG | <https://github.com/Ren9618/MacUtl/releases/download/v0.8.11-mac.1/MacUtl-0.8.11-macos.dmg> |
| ZIP | <https://github.com/Ren9618/MacUtl/releases/download/v0.8.11-mac.1/MacUtl-0.8.11-macos.zip> |
| Issues | <https://github.com/Ren9618/MacUtl/issues> |
| 上流 NeoUtl | <https://codeberg.org/taisho-guy/NeoUtl> |

### 2.2 リリース成果物

| ファイル | サイズ | SHA-256 |
|---|---:|---|
| `MacUtl-0.8.11-macos.dmg` | 40,814,500 bytes | `b08ec088ccf18d33f59688add3d763aa2605a9653f363a28dcfc3b4e9aa05548` |
| `MacUtl-0.8.11-macos.zip` | 36,332,190 bytes | `d1d689d03cf0f1a735d182fe9c27b8d1bd4f6b2e3fa5e406c716bc0619252aba` |

- リリース名: 「MacUtl 0.8.11 (macOS fork 初回リリース)」
- 公開日時: 2026-11-12 19:12:19 JST（`2026-11-12T10:12:19Z`）
- GitHub の asset digest（`sha256:`）は上記ローカル値と一致。
- 成果物は `target/bundle/` からアップロードし、リリース本文のチェックサムも同じ値に更新した。

### 2.3 タグと公開コミット

- `v0.8.11-mac.1` は annotated tag。タグオブジェクトは `3e3cdc0148dd9c1d6c72a46c620fb5f2f39c01f9`。
- タグが指すコミットは `eae56b199c24011cfd06a9ce3683a1dc50b34052`。
- 本報告書はタグ公開後に `main` へ追加するため、タグは移動しない。タグ時点のソースには本報告書は含まれないが、監査・変更通知・リリースノートは含まれる。

---

## 3. ライセンス監査

詳細は `LICENSE_AUDIT.md` に記録した。以下は要約。

### 3.1 Rust クレート

| 項目 | 値 |
|---|---:|
| Cargo.lock 内の総パッケージ数 | 792 |
| ワークスペースメンバー | 58 |
| 外部クレート（全ターゲット） | 734 |
| macOS arm64 の通知対象エントリ | **508** |
| 個別ライセンス本文グループ | **142** |
| ライセンス概要の種類 | 16 |

- `cargo about` は `about.toml` の accepted リストに基づき exit 0。ライセンス未記載の外部クレートは 0 件。
- 単一ライセンスが GPL-2.0-only のクレートは 0 件。
  - `self_cell 1.3.0` は `Apache-2.0 OR GPL-2.0-only` のデュアルライセンス。**Apache-2.0** を選択。
  - `r-efi 5.3.0 / 6.0.0` は `MIT OR Apache-2.0 OR LGPL-2.1-or-later`。**MIT** を選択。
- MPL-2.0 のクレートは 11 件。MPL-2.0 は AGPL-3.0 と互換。
- `Unicode-3.0`、`OFL-1.1`、`Ubuntu-font-1.0`、`CC0-1.0` などの許諾ライセンスのみ。
- 生成物 `THIRD_PARTY_NOTICES.md` はリポジトリ直下と `MacUtl.app/Contents/Resources/` の双方に同梱。

### 3.2 FFmpeg と同梱ネイティブライブラリ

Homebrew FFmpeg 9.0.2 の `ffmpeg -buildconf` で実測した configure フラグ:

```text
--enable-gpl
--enable-version3
--enable-libx264
--enable-libx265
（--enable-nonfree は無し）
```

| 項目 | 判定 |
|---|---|
| FFmpeg のライセンス | **GPL-3.0-or-later** |
| AGPL-3.0-or-later との両立 | **可**（GPLv3 第 13 条） |
| Homebrew formula / SBOM | `GPL-3.0-or-later` を宣言・一致 |
| x264 / x265 | GPL-2.0-or-later（GPLv3 を選択して同梱） |
| SVT-AV1 / dav1d / libvpx / libvmaf / Opus | BSD 系 |
| OpenSSL | Apache-2.0（GPLv3 と互換） |
| LAME | LGPL-2.0-or-later |
| mpg123 | LGPL-2.1-only（LGPLv2.1 §3 により GPLv3 と組み合わせ可能、未改変） |
| XZ / liblzma | 0BSD |

- 対応ソース: 本リポジトリと <https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz>（SHA-256 `8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e`）。
- `third_party/licenses/` に FFmpeg、x264、x265、SVT-AV1、dav1d、libvpx、libvmaf、OpenSSL、LAME、mpg123、Opus、XZ の本文を保存。
- `THIRD_PARTY_NOTICES.md` に Independent JPEG Group へのクレジットを記載。

### 3.3 branding・フォント・i18n

- `assets` サブモジュール（`neoutl-branding`）の `README.md` と `LICENSE` を確認し、**CC0 1.0** であることを確認。上流アイコンの継続使用・改変・再配布は許諾されている。
- MacUtl ではアイコンを `MacUtl.icns` として再生成。アイコンは抽象的なフィルム／N モチーフで、製品名の誤認を招く文字は含まない。
- フォントは `egui_system_fonts` による macOS システムフォントを使用し、フォントファイルはバンドルしない。
- テーマはコード内定義で、外部テーマファイルは同梱しない。
- `i18n` サブモジュール（`neoutl-locales`）は **CC0 1.0**。プラグイン同梱の `.yml` も同じ CC0 素材。
- Slang / DXC はビルド時のみ使用し、`.app` には同梱しない。

### 3.4 AGPL チェックリスト

| 項目 | 状態 | 証跡 |
|---|---|---|
| `LICENSE` がリポジトリ直下にある | 済 | `LICENSE`（AGPL-3.0） |
| `LICENSE` が `.app/Contents/Resources` に入る | 済 | パッケージスクリプトで同梱 |
| 変更点と変更日を示す文書 | 済 | `MACOS_CHANGES.md`（2026-11-12） |
| 変更の証跡 | 済 | `MACOS_PHASE0_1_REPORT.md` 〜 本報告書 |
| 第三者ライセンス一覧 | 済 | `THIRD_PARTY_NOTICES.md`（Rust 508 エントリ + ネイティブ 12 種） |
| 第三者ライセンス本文 | 済 | `third_party/licenses/`、`.app` 内にも同梱 |
| UI 上のライセンス表示 | 代替済 | About 画面は無い。README とアプリ内 `LICENSE` で明記 |
| 対応するソースの提供 | 済 | GitHub リポジトリ + FFmpeg ソース URL / SHA-256 |
| 上流の著作権表示の保持 | 済 | `LICENSE`、README のクレジット、git 履歴を保持 |
| 公式版と誤認させない表記 | 済 | README / Release に「非公式フォーク」と明記 |

### 3.5 名前の一次確認

2026-11-12 に Web 検索を実施。

- `MacUtl macOS 動画編集 ソフト`、`"MacUtl" app`、`"MacUtl" GitHub`、`"まっくうとる"` 系のいずれも、現行の動画編集ソフト・Mac アプリに該当なし。
- 1991 年の Classic Mac 用アーカイブ名 `macutl.sit` の言及のみ確認。現行ソフトではなく、商標的な衝突は一次調査では確認されなかった。
- 金融機関の「MACU」とは分野・表記が異なる。
- **J-PlatPat による正式な商標調査は未実施**。必要に応じてユーザーが別途行う前提。

### 3.6 ゲート判定

| ゲート項目 | 判定 |
|---|---|
| Rust クレートに AGPL 非互換ライセンスが無い | **PASS** |
| FFmpeg が再配布可能で AGPL と両立する | **PASS**（GPL-3.0-or-later） |
| branding 素材の再配布許諾が明確 | **PASS**（CC0-1.0） |
| フォント・テーマ・i18n の権利が明確 | **PASS**（システムフォント / CC0） |
| AGPL の LICENSE・変更通知・ソース提供 | **PASS** |
| 名前の一次確認 | **PASS（一次のみ）** |

**公開ブロッカーは無い。**

---

## 4. ネットワーク監査

### 4.1 検出された外部エンドポイント

ソースツリー（`src/`、`crates/`、`xtask/`、`scripts/`、`Cargo.toml`）を静的走査した結果、アプリ本体が参照する外部エンドポイントは次の 2 系統のみだった。

| 用途 | エンドポイント | macOS での挙動 | 実装 |
|---|---|---|---|
| 上流の更新チェック | `https://codeberg.org/api/v1/repos/taisho-guy/NeoUtl` | **無効**（常に `UpToDate`、スレッドを起動しない） | `src/app/update.rs` の `spawn_check` が `#[cfg(target_os = "macos")]` で早期 return |
| クラッシュレポート | 上流の GlitchTip / Sentry DSN | **無効**（Sentry クライアントを初期化しない） | `src/infra/crash_report.rs` の `init` が macOS で `None` を返す |

- `spawn_apply` も macOS では自己置換を行わず、エラーメッセージを返すだけ。
- クラッシュレポートの DSN 定数はソースに残っているが、macOS では `config::SENTRY_DSN` の import 自体が `#[cfg(not(target_os = "macos"))]` で除外される。リリースバイナリを `strings` で確認し、DSN の URL・トークンが含まれないことを確認した。
- 上記以外にテレメトリ、利用統計、広告 SDK、ライセンスサーバへの問い合わせは無い。
- VST3 プラグインのスキャンはローカルファイルシステムのみを対象とする。
- README / Release 内の外部リンクは、ユーザーがブラウザで開いた場合にのみ接続する。

### 4.2 制約

- 本監査はコードとリリースバイナリの静的確認であり、パケットキャプチャによる全通信の証明ではない。
- 上流コードの非 macOS ビルドには、従来どおり更新チェックとクラッシュレポートが残る。MacUtl の macOS 配布物では無効化されている。

---

## 5. リネーム

### 5.1 変更したユーザー可視面

| 対象 | 変更後 |
|---|---|
| アプリバンドル | `MacUtl.app` |
| `CFBundleName` / `CFBundleDisplayName` | `MacUtl` |
| `CFBundleIdentifier` | `works.rayno.macutl` |
| バンドル内メイン実行ファイル | `Contents/MacOS/MacUtl` |
| ウィンドウタイトル | `MacUtl` / `MacUtl - <プロジェクト名>` |
| ランタイムデータ | `~/Library/Application Support/MacUtl` |
| アイコン | `MacUtl.icns` |
| 配布物 | `MacUtl-0.8.11-macos.dmg` / `.zip` |
| パッケージスクリプト | `scripts/package_macos.py` / `.sh`（`cargo xtask package --release` から呼び出し） |
| 対応ソース案内 | `Contents/Resources/SOURCE.txt` |

### 5.2 意図的に変更しなかったもの

- 内部クレート名（`NeoUtl`、`neoutl-*`）、ソースモジュール名、プラグイン dylib 名。
- `.npb` プロジェクト形式と NeoUtl プラグイン ABI。
- 上流の `LICENSE`、著作権表示、git 履歴。
- 上流 README は `README.upstream.md` として保存し、表示される `README.md` を MacUtl 用に差し替えた。
- 「NeoUtl」はクレジット、上流参照、内部識別子として意図的に残している。

### 5.3 ビルド再現用の i18n パッチ

- `cargo xtask build` は上流の仕様で `i18n/ja.yml` を再生成する。このファイルは `i18n` サブモジュールの作業ツリーに属するため、MacUtl 側ではコミットしない。
- リリース版と同一の日本語表示を再現するため、差分を `patches/i18n-macutl-ja.patch` として保存した。
- サブモジュールの固定コミットは変更していない。

---

## 6. ビルドとバンドル検証

### 6.1 ビルド

- ビルド日時: 2026-11-12 18:50 JST
- コマンド: `cargo xtask package --release`
- 出力: `target/bundle/MacUtl.app`、`target/bundle/MacUtl-0.8.11-macos.dmg`、`.zip`
- 署名: `codesign --sign -`（ad-hoc、内側から順に署名）
- Apple ID / Developer ID / notarization / `sudo` は未使用。

### 6.2 バンドル構成

| 項目 | 値 |
|---|---|
| `.app` サイズ | 約 90 MB（ディスク上 92,076 KB） |
| メイン実行ファイル | 31,130,880 bytes / SHA-256 `b72e9f7af95806b7db4bfd09da04f06d3f6596d18f97379eb8e05116e5a73c45` |
| アーキテクチャ | `Mach-O 64-bit executable arm64`（thin） |
| Frameworks | Homebrew 由来 dylib 27 ファイル |
| Objects | 9 プラグイン |
| Effects | 23 プラグイン |
| Decoders | 2 プラグイン dylib（+ FFmpeg 内蔵デコーダで UI 上 3 種） |
| Easing | 1 種（コード内蔵） |
| ライセンス本文 | `third_party/licenses/` 12 ディレクトリ / 18 ファイル |
| 同梱ドキュメント | `LICENSE`、`LICENSE_AUDIT.md`、`MACOS_CHANGES.md`、`THIRD_PARTY_NOTICES.md`、`SOURCE.txt` |

### 6.3 署名・Info.plist

```text
Identifier=works.rayno.macutl
Format=app bundle with Mach-O thin (arm64)
Signature=adhoc
TeamIdentifier=not set
```

- `codesign --verify --deep --strict` 合格。
- `CFBundleName` / `CFBundleDisplayName`: `MacUtl`
- `CFBundleExecutable`: `MacUtl`
- `CFBundleIdentifier`: `works.rayno.macutl`
- `CFBundleShortVersionString`: `0.8.11`
- `CFBundleVersion`: `1182`
- `LSMinimumSystemVersion`: `27.0`
- メイン実行ファイルと全 dylib の install name を確認し、`/opt/homebrew` や `/usr/local` の絶対パスが残っていないことを確認。
- `spctl` は ad-hoc 署名のため拒否する（想定どおり）。Gatekeeper の回避手順は README と Release に記載。

---

## 7. 回帰試験

### 7.1 コード品質

| コマンド | 結果 |
|---|---|
| `cargo fmt --all -- --check` | 合格（exit 0、差分 0） |
| `cargo test --workspace --exclude NeoUtl` | **80 件の `test result: ok`**、failed 0、ignored 7（doctest） |

- テストログ: `target/phase45/logs/cargo_test_final2.log`
- Phase 4.5 の lost-wakeup 修正を含む最終ソースに対する結果。

### 7.2 720p 動画＋画像書き出し（パッケージ済み `.app`）

| 項目 | 結果 |
|---|---|
| プロジェクト | `phase2_e2e_720p`（動画 + 画像 + 音声） |
| 実行環境 | `target/bundle/MacUtl.app` の通常 UI |
| 出力 | `target/phase5/exports/macutl_pkg_video_image_720p.mp4` |
| フレーム数 | **391** |
| 解像度 / コーデック | 1280×720 / H.264 |
| 尺 | 13.000 秒 |
| ファイルサイズ | 79,360 bytes |
| SHA-256 | `bccf52e6a5424a1feea36614ebc451ed77df0afd598f409099d8af807d2fab2b` |
| クラッシュ | 0 |

- このハッシュは Phase 4.5 の 391 フレーム基準出力と一致。
- 動画＋画像の同時書き出しで発生していたフレーム 43→44 の停止は再発していない。

### 7.3 既存ベースライン

- Phase 4.5 で 1080p / 60 秒（1801 フレーム）は 37 秒で完走（Phase 2 の 36.90 秒と同等）。
- Phase 5 の変更は主にパッケージング、名称、ドキュメントであり、レンダリング経路の変更は Phase 4.5 修正を含むコミット `0eef8d56` で確定している。
- 起動時フォントプリロードの一時メモリ増加、VRAM/RAM 予算取得失敗時のフォールバックは既知の別課題として残る（第 10 節）。

---

## 8. リリース済み成果物の再ダウンロード検証

公開後に GitHub Releases から DMG / ZIP を再取得し、配布物そのものを検証した。

### 8.1 取得とハッシュ

| ファイル | サイズ | SHA-256 | ローカル成果物との一致 |
|---|---:|---|---|
| `target/phase5/download/MacUtl-0.8.11-macos.dmg` | 40,814,500 bytes | `b08ec088ccf18d33f59688add3d763aa2605a9653f363a28dcfc3b4e9aa05548` | 一致 |
| `target/phase5/download/MacUtl-0.8.11-macos.zip` | 36,332,190 bytes | `d1d689d03cf0f1a735d182fe9c27b8d1bd4f6b2e3fa5e406c716bc0619252aba` | 一致 |

- `gh release download v0.8.11-mac.1` で取得。HTTPS 経由のため `com.apple.quarantine` 属性は付与されていなかった。
- GitHub の asset digest とも一致。

### 8.2 DMG の起動検証

1. ダウンロードした DMG を `/Volumes/MacUtl` にマウント。
2. マウント内の `MacUtl.app` を起動（PID 64982）。
3. `codesign --verify --deep --strict` 合格、Identifier が `works.rayno.macutl` であることを確認。
4. 通常 UI から 720p 動画＋画像プロジェクトを書き出し。
5. 出力 `target/phase5/exports/post_download_720p.mp4` は 391 フレーム / 1280×720 / 13.000 秒 / 79,360 bytes。
6. SHA-256 は `bccf52e6a5424a1feea36614ebc451ed77df0afd598f409099d8af807d2fab2b` で、パッケージ済み `.app` の出力および Phase 4.5 基準と一致。
7. アプリを終了し、DMG をアンマウント。

### 8.3 ZIP の検証

1. ダウンロードした ZIP を `target/phase5/verify_zip_download/` に展開。
2. `MacUtl.app` の `codesign --verify --deep --strict` 合格。
3. バンドル構成と Info.plist が DMG 版と一致することを確認。

### 8.4 判定

**リリース済み成果物は、公開前のローカル成果物と同一であり、起動・署名・書き出しのいずれも合格。**

---

## 9. コミット・リモート

### 9.1 `main` のコミット

| コミット | 内容 |
|---|---|
| `0eef8d56` | macOS 移植: VideoToolbox / IME / `.app` ランタイム / 書き出し修正 |
| `d018b27a` | MacUtl へのリネームと macOS パッケージング |
| `2666b480` | ドキュメントとライセンス監査 |
| `6e15951f` | ビルド再現用の i18n パッチと未使用 import 修正 |
| `eae56b19` | README 用スクリーンショットとリリースノートのハッシュを追加 |

- 上流の履歴は書き換えていない。タグ時点で `0ead3935` の上に 5 コミットを積んでいる。
- `origin` は Codeberg の上流のまま。**Codeberg への push は行っていない。**
- 公開に使用したリモートは `github`（`git@github.com:Ren9618/MacUtl.git`）のみ。
- force push は行っていない。タグ `v0.8.11-mac.1` は `eae56b19` を指したまま固定する。

### 9.2 リポジトリ状態

- GitHub リポジトリは public。
- `main` は `eae56b19` で公開済み。本報告書の追加コミットはその後に積む。
- `i18n` サブモジュールは `cargo xtask build` による `ja.yml` 再生成で dirty になるが、これは上流仕様どおり。MacUtl 側ではコミットしない。
- `target/`、DMG / ZIP、認証情報はコミットしていない。

---

## 10. 残る課題

1. **Apple Developer ID 署名・公証が無い**
   - ad-hoc 署名のため Gatekeeper の警告が出る。README と Release に右クリック→「開く」等の手順を記載済み。
   - 配布規模を広げる場合は Developer ID 署名と notarization を検討する。

2. **対応環境が限定的**
   - Apple Silicon（arm64）thin のみ。Intel Mac / Universal2 は配布していない。
   - macOS 27.0 以降。同梱 FFmpeg 9.0.2 の dylib が `minos 27.0` のため。

3. **上流がプレアルファ**
   - API・プロジェクト形式・UI は予告なく変わる可能性がある。

4. **VST3 の実適用は未確認**
   - スキャン確認のみ。音声処理への適用は今後の検証課題。

5. **自動更新は無い**
   - macOS では `.app` の自己置換が署名を壊すため無効化。新版は Releases から手動取得。

6. **本家 AviUtl プラグインとのバイナリ互換は無い**
   - NeoUtl のオブジェクト／エフェクト ABI を使用する。

7. **プラグイン dylib 間の wgpu / parking_lot 重複**
   - Phase 4.5 の共有ロック直列化は macOS 向けの応急・最小修正。プラグインが増えるとロック競合と保守コストが増す。
   - 本命の解決は、ホストの GPU ランタイムを共有するプラグイン ABI の設計。

8. **起動時フォントプリロードのメモリ増加**
   - システムフォントを一括プリロードするため、起動直後に一時的な high-water mark が上がる。Phase 5 の公開ブロッカーではない。

9. **VRAM/RAM 予算取得失敗時のフォールバック**
   - 720p 全編再生の RSS は 64 フレームの RAM キャッシュに支配される場合がある。既存挙動。

10. **商標の正式調査**
    - Web 検索による一次確認のみ。J-PlatPat 等の正式調査は未実施。

11. **FFmpeg の対応ソース提供は運用で維持**
    - GPLv3 のバイナリ配布義務として、リポジトリと FFmpeg ソース URL / SHA-256 を維持する。

12. **GlitchTip の DSN 文字列がソースに残る**
    - macOS では初期化されず、バイナリにも DSN の URL は含まれない。上流コードとの差分を小さく保つため、定数自体は残している。

---

## 11. 付録

### 11.1 主要ファイル

| ファイル | 内容 |
|---|---|
| `README.md` | MacUtl の導入・Gatekeeper・プライバシ・ライセンス・ビルド手順 |
| `README.upstream.md` | 上流 NeoUtl の README（保存） |
| `LICENSE` | AGPL-3.0 全文（上流から保持） |
| `LICENSE_AUDIT.md` | ライセンス本調査と公開ゲート |
| `THIRD_PARTY_NOTICES.md` | Rust 508 エントリ + ネイティブライブラリの通知 |
| `MACOS_CHANGES.md` | AGPL 変更通知とリネーム一覧 |
| `RELEASE_NOTES_v0.8.11-mac.1.md` | 公開したリリース本文 |
| `scripts/package_macos.py` | `.app` / DMG / ZIP の生成、署名、同梱 |
| `patches/i18n-macutl-ja.patch` | i18n サブモジュールの再現用差分 |
| `docs/screenshots/macutl-main.png` | README 用スクリーンショット |

### 11.2 検証用成果物（コミット対象外）

| パス | 内容 |
|---|---|
| `target/bundle/MacUtl.app` | リリース版アプリ |
| `target/bundle/MacUtl-0.8.11-macos.dmg` | リリース版 DMG |
| `target/bundle/MacUtl-0.8.11-macos.zip` | リリース版 ZIP |
| `target/phase5/exports/macutl_pkg_video_image_720p.mp4` | パッケージ済み `.app` の 720p 出力 |
| `target/phase5/exports/post_download_720p.mp4` | 再ダウンロード版の 720p 出力 |
| `target/phase5/download/` | 再ダウンロードした DMG / ZIP |

### 11.3 参照

- 上流 NeoUtl: <https://codeberg.org/taisho-guy/NeoUtl>
- 上流 v0.8.11 コミット: `0ead39350db55996810de53da79d02fbc98355f5`
- MacUtl リポジトリ: <https://github.com/Ren9618/MacUtl>
- リリース: <https://github.com/Ren9618/MacUtl/releases/tag/v0.8.11-mac.1>

---

**MacUtl は非公式フォークであり、本家 AviUtl および NeoUtl の公式版・移植版ではありません。上流の作者とは提携・承認関係にありません。**
