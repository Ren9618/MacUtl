# MacUtl ライセンス本調査（Phase 5 公開ゲート）

- **調査日**: 2026-11-12
- **対象**: MacUtl 0.8.11（macOS fork、ベース: NeoUtl v0.8.11 / commit `0ead39350db55996810de53da79d02fbc98355f5`）
- **調査環境**: macOS 27.0.1 (Build 26A434) / Apple M1 (arm64) / Rust 1.99.1 / cargo-about 0.9.3
- **判定**: **PASS（公開ブロッカーなし）**

本書は法的意見ではない。AGPL-3.0-or-later の公開に必要な実務チェックを、実物の依存関係・
バイナリ・素材について記録したものである。J-PlatPat 等による商標の正式確認はユーザーが
別途行う前提であり、本書では Web 検索による一次確認までを実施した。

---

## 1. 調査方法と再現コマンド

### 1.1 Rust クレート全量

`cargo metadata` で依存グラフを取得し、`cargo about` で macOS（arm64）リリースに
含まれるクレートのライセンス本文を生成した。`cargo about` は Cargo.lock と
`about.toml` の accepted リストに基づき、ライセンス不明・未承認のクレートがあれば
非ゼロ終了する。

```bash
cargo metadata --format-version 1 --all-features > /tmp/neoutl_metadata.json
cargo about generate --offline --target aarch64-apple-darwin \
  scripts/about.hbs -o target/rust_third_party.md
sh scripts/generate_third_party_notices.sh
```

結果:

| 項目 | 値 |
|---|---:|
| Cargo.lock 内の総パッケージ数 | 792 |
| ワークスペースメンバー | 58 |
| 外部クレート（全ターゲット） | 734 |
| macOS arm64 の通知対象エントリ | **508** |
| 個別ライセンス本文グループ | **142** |
| ライセンス概要の種類 | 16 |

生成物は `THIRD_PARTY_NOTICES.md`（13,119 行）としてリポジトリ直下に置き、
`MacUtl.app/Contents/Resources/THIRD_PARTY_NOTICES.md` にも同梱する。

### 1.2 判定結果

- `about.toml` の accepted リストで全クレートが承認された（`cargo about` exit 0）。
- ライセンス未記載（`license` も `license_file` も無い）外部クレートは **0 件**。
- 単一ライセンスが GPL-2.0-only のクレートは **0 件**。
  - `self_cell 1.3.0` は `Apache-2.0 OR GPL-2.0-only` のデュアルライセンス。
    MacUtl は **Apache-2.0** を選択する（GPL-2.0-only を選択しない）。
  - `r-efi 5.3.0 / 6.0.0` は `MIT OR Apache-2.0 OR LGPL-2.1-or-later`。
    MacUtl は **MIT** を選択する（macOS ビルドでは実質未使用）。
- `MPL-2.0` のクレートは 11 件あるが、MPL-2.0 は AGPL-3.0 と互換であり、
  ソース公開の条件下で同梱可能。
- `Unicode-3.0`、`OFL-1.1`、`Ubuntu-font-1.0`、`CC0-1.0` などの許諾ライセンスのみ。
- `--enable-nonfree` に相当する非自由ライセンスは Rust 側にも FFmpeg 側にも無い。

### 1.3 AGPL との両立確認

AGPL-3.0-or-later と両立しないライセンス（GPL-2.0-only 単独、proprietary、
non-commercial 等）は検出されなかった。デュアルライセンスはすべて許諾側を選択した。
GPL-3.0 / LGPL-2.1 / MPL-2.0 / Apache-2.0 は AGPL-3.0-or-later と共存できる。
GPLv3 と AGPLv3 の組み合わせは GPLv3 第 13 条で明示的に許容されている。

---

## 2. FFmpeg のライセンス確定

### 2.1 実測した configure フラグ

`ffmpeg -buildconf`（Homebrew FFmpeg 9.0.2）の原文:

```text
configuration:
  --prefix=/opt/homebrew/Cellar/ffmpeg/9.0.2
  --enable-shared
  --enable-pthreads
  --enable-version3
  --cc=clang
  --host-cflags=
  --host-ldflags=
  --enable-ffplay
  --enable-gpl
  --enable-libsvtav1
  --enable-libopus
  --enable-libx264
  --enable-libmp3lame
  --enable-libdav1d
  --enable-libvmaf
  --enable-libvpx
  --enable-libx265
  --enable-openssl
  --enable-videotoolbox
  --enable-audiotoolbox
  --enable-neon
```

### 2.2 判定

| 項目 | 結果 |
|---|---|
| `--enable-gpl` | **あり**（GPL コンポーネントを有効化） |
| `--enable-version3` | **あり**（GPLv3 を選択） |
| `--enable-nonfree` | **なし**（再配布可能） |
| 同梱 FFmpeg のライセンス | **GPL-3.0-or-later** |
| AGPL-3.0-or-later との両立 | **可**（GPLv3 §13） |

Homebrew の formula も `license "GPL-3.0-or-later"` を宣言しており、SBOM
（`/opt/homebrew/Cellar/ffmpeg/9.0.2/sbom.spdx.json`）の `licenseConcluded` も
`GPL-3.0-or-later` である。

### 2.3 同梱ネイティブライブラリのライセンス

| コンポーネント | バージョン | ライセンス | 判定 |
|---|---:|---|---|
| FFmpeg（libav* / libsw*） | 9.0.2 | GPL-3.0-or-later | 可（GPLv3 §13） |
| x264 | r3222 | GPL-2.0-or-later | 可（GPLv3 を選択） |
| x265 | 4.3 | GPL-2.0-or-later | 可（GPLv3 を選択） |
| SVT-AV1 | 4.2.0 | BSD-3-Clause | 可 |
| dav1d | 1.5.4 | BSD-2-Clause | 可 |
| libvpx | 1.17.0 | BSD-3-Clause | 可 |
| libvmaf | 3.2.1 | BSD-2-Clause-Patent | 可 |
| OpenSSL（libssl/libcrypto） | 3.6.4 | Apache-2.0 | 可（GPLv3 と互換） |
| LAME（libmp3lame） | 4.0 | LGPL-2.0-or-later | 可 |
| mpg123 | 1.33.7 | LGPL-2.1-only | 可（LGPLv2.1 §3 により GPLv3 を選択） |
| Opus | 1.6.1 | BSD-3-Clause | 可 |
| XZ / liblzma | 5.8.4 | 0BSD（liblzma） | 可 |

- mpg123 の LGPL-2.1-only は、LGPLv2.1 第 3 条が GPLv2 またはそれ以降の
  GPL への切り替えを認めているため、GPLv3 の FFmpeg と組み合わせられる。
  mpg123 のソースは改変していない。
- x264 / x265 は GPL-2.0-or-later であり、GPLv3 の FFmpeg 内で GPLv3 として
  利用される。AGPL-3.0-or-later と両立する。
- `third_party/licenses/` に FFmpeg の `LICENSE.md`、`COPYING.GPLv3`、
  `COPYING.GPLv2`、`COPYING.LGPLv2.1`、x264 / x265 / mpg123 / LAME / OpenSSL /
  BSD 系の本文を保存した。
- FFmpeg の `LICENSE.md` が要求する Independent JPEG Group へのクレジットを
  `THIRD_PARTY_NOTICES.md` に記載した。

### 2.4 対応するソースの提供

- MacUtl の完全なソース: <https://github.com/Ren9618/MacUtl>
- 同梱 FFmpeg は Homebrew の未改変ビルド。対応ソースは
  <https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz>
  （SHA-256 `8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e`）。
- Homebrew のビルドレシピ:
  <https://github.com/Homebrew/homebrew-core/blob/master/Formula/f/ffmpeg.rb>
- 第三者のソース提供が必要な場合は GitHub Issues で連絡を受け付ける。

---

## 3. branding 素材

### 3.1 実物確認

`assets` サブモジュール（`neoutl-branding`、commit
`0ab51ecdc00569cdff0170fd58f50c3a688d09bf`）の `README.md` と `LICENSE` を確認:

```text
assets/README.md: ## ライセンス / [CC0 1.0](...)
assets/LICENSE:   Creative Commons Legal Code / CC0 1.0 Universal
```

- **CC0 1.0** は著作権を放棄し、再配布・改変・商用利用を明示的に許諾する。
- したがって本家アイコン（`assets/neoutl/icon.svg`）の継続使用は許諾されている。
- MacUtl ではアイコンを `MacUtl.icns` として再生成して使用する。
  アイコン画像はテキストを含まない抽象的なフィルム／N モチーフであり、
  MacUtl の製品名表示と混同しない。
- スプラッシュ画像（`splash.svg` / `splash.webp`）は現行コードから参照されて
  おらず、バンドルにも含まれない。ロゴ画像を UI に表示する箇所は無い。
- About 画面は実装されていない（`grep -rn "About" src/` で該当なし）。
  ライセンス表示は README とアプリ内 `Contents/Resources/LICENSE` で代替する。

### 3.2 フォント・テーマ・i18n

- フォント: `egui_system_fonts` により macOS のシステムフォントを利用する。
  フォントファイルはバンドルしない。したがってフォントの再配布は発生しない。
- テーマ: アプリ内のコードで定義され、外部テーマファイルはバンドルしない。
- i18n: `i18n` サブモジュール（`neoutl-locales`、commit
  `f546a55359dc18c3f6378977527a242029d3171d`）は **CC0 1.0**。
  `objects` / `effects` の各プラグインに同梱される `.yml` も同じ CC0 素材である。
- プラグイン dylib 内のシェーダ・コードは MacUtl / NeoUtl の AGPL コードである。
- 外部から取得する Slang / DXC はビルド時のみ使用し、.app には同梱しない。

---

## 4. AGPL チェックリスト

| 項目 | 状態 | 証跡 |
|---|---|---|
| `LICENSE` 本文がリポジトリ直下にある | **済** | `LICENSE`（AGPL-3.0、35 KB） |
| `LICENSE` が .app の `Contents/Resources` に入る | **済** | パッケージスクリプトで同梱（Phase 5 で実装） |
| 変更点と変更日を示す文書 | **済** | `MACOS_CHANGES.md`（2026-11-12、Phase 0〜4.5） |
| 変更の証跡（Phase 報告書） | **済** | `MACOS_PHASE0_1_REPORT.md` 〜 `MACOS_PHASE5_REPORT.md` |
| 第三者ライセンス一覧 | **済** | `THIRD_PARTY_NOTICES.md`（Rust 508 エントリ + ネイティブ 12 種） |
| 第三者ライセンス本文 | **済** | `third_party/licenses/`、.app 内にも同梱 |
| UI 上のライセンス表示 | **代替済** | About 画面は無い。README とアプリ内 LICENSE で明記 |
| 対応するソースの提供 | **済** | GitHub リポジトリ + FFmpeg ソース URL / SHA-256 |
| 上流の著作権表示の保持 | **済** | `LICENSE`、README のクレジット、git 履歴を保持 |
| 公式版と誤認させない表記 | **済** | README / Release に「非公式フォーク」と明記 |

### 4.1 アプリ内の同梱物

`MacUtl.app/Contents/Resources/` に以下を入れる（Phase 5 のパッケージ処理）:

```text
LICENSE
THIRD_PARTY_NOTICES.md
MACOS_CHANGES.md
third_party/licenses/...
```

これにより、バイナリ配布物単体でも AGPL のライセンス本文・変更通知・
第三者通知が揃う。

---

## 5. 名前の一次確認（Web 検索）

2026-11-12 に以下を検索した。

| 検索語 | 結果 |
|---|---|
| `MacUtl macOS 動画編集 ソフト` | 該当する現行ソフトなし（iMovie / Shotcut 等の一般記事のみ） |
| `"MacUtl" app` | 該当なし（MACU は金融機関のアプリで無関係） |
| `"MacUtl" GitHub` | 現行ソフトなし。1991 年の Classic Mac 用アーカイブ名 `macutl.sit` の言及のみ |
| `"まっくうとる" ソフト` | 該当なし |
| `"まっくうとる" OR "マックウトル"` | 該当なし（人物・商品の類似表記のみ） |

- 現行の動画編集ソフト・Mac アプリに「MacUtl」は見つからなかった。
- 1991 年の `macutl.sit` は現行ソフトではなく、商標的な衝突は一次調査では
  確認されなかった。
- **J-PlatPat の正式確認は未実施**。ユーザーが別途行う前提。
- 類似名の「MACU」（金融）とは分野・表記が異なる。

---

## 6. ゲート判定

| ゲート項目 | 判定 |
|---|---|
| Rust クレートに AGPL 非互換ライセンスが無い | **PASS** |
| FFmpeg が再配布可能で AGPL と両立する | **PASS**（GPL-3.0-or-later） |
| branding 素材の再配布許諾が明確 | **PASS**（CC0-1.0） |
| フォント・テーマ・i18n の権利が明確 | **PASS**（システムフォント / CC0） |
| AGPL の LICENSE・変更通知・ソース提供 | **PASS**（Phase 5 で同梱） |
| 名前の一次確認 | **PASS（一次のみ）** |

**ブロッカーは無い。Phase 5 の公開作業に進んでよい。**

### 残る留意点

1. 本書は法的助言ではない。商用配布や大規模配布の前に、必要なら弁護士の確認を受ける。
2. FFmpeg は GPLv3 であり、MacUtl のバイナリ配布時には対応ソースの提供義務がある。
   本リポジトリと上記 URL で満たす運用とする。
3. J-PlatPat による「MacUtl」の商標調査は別途必要。
4. 上流 NeoUtl の README・著作権表示は削除せず、フォークであることを明記する。
