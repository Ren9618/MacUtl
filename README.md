# MacUtl

**NeoUtl をベースに開発した、macOS ネイティブの AviUtl 風動画編集ソフトです。本家 AviUtl および NeoUtl の公式版・移植版ではありません（非公式フォーク）。**

- ベース: [NeoUtl](https://codeberg.org/taisho-guy/NeoUtl) v0.8.11（プレアルファ、コミット `0ead393`）
- 対応: Apple Silicon（arm64）のみ / macOS 27.0 以降
- ライセンス: AGPL-3.0-or-later
- 配布: [GitHub Releases](https://github.com/Ren9618/MacUtl/releases)

![MacUtl のメイン画面](docs/screenshots/macutl-main.png)

## 目次

- [MacUtl とは](#macutl-とは)
- [VOICEVOX 連携（Phase 8）](#voicevox-連携phase-8)
- [動作環境](#動作環境)
- [インストール](#インストール)
- [既知の制限](#既知の制限)
- [プライバシ](#プライバシ)
- [ライセンス](#ライセンス)
- [作者の他のソフト](#作者の他のソフト)
- [サポート](#サポート)
- [ソースからビルド](#ソースからビルド)
- [謝辞](#謝辞)

## MacUtl とは

MacUtl は、AviUtl ExEdit 風のタイムライン編集・プレビュー・書き出しを macOS 上で行えるようにした非公式フォークです。NeoUtl の Rust / wgpu 実装をベースに、YMM4 風の一画面レイアウト、macOS 向けの VideoToolbox ハードウェアデコード、日本語 IME 入力、`.app` パッケージング、Gatekeeper を考慮した配布形態を追加しています。

本家 AviUtl のプラグイン（`.auf` / `.aui` / `.auo` 等）とのバイナリ互換はありません。NeoUtl のオブジェクト／エフェクトプラグイン ABI を使用します。

### 現在できること（Phase 8 時点で実測済み）

- 起動、プロジェクト作成・保存・読み込み
- 一画面ワークスペース（プレビュー／プロパティ／タイムライン）。境界ドラッグによるサイズ変更、ペインの入れ替え、レイアウトの自動保存・復元
- タイムライン編集、Undo / Redo
- プレビュー（VideoToolbox HW デコード、失敗時はソフトウェアにフォールバック）
- 日本語表示と IME による直接入力
- Objects 9 種 / Effects 23 種 / Decoders 3 種 / Easing 1 種の登録
- 720p / 1080p の H.264 MP4 書き出し（動画＋画像プロジェクトを含む）。音声付きプロジェクトでは AAC 音声トラックを混合して書き出します
- VOICEVOX 連携（セリフ入力 → 音声＋字幕の自動配置、保存 / 読込、音声付き書き出しまで実測）
- 音声プラグイン（VST3）のスキャン（実適用は未確認）

### 一画面レイアウト（Phase 7）

プレビュー・プロパティ・タイムラインを 1 ウィンドウにまとめた YMM4 風のレイアウトです。既定は「上段: プレビュー＋プロパティ / 下段: タイムライン全幅」で、従来に近い配置にも切り替えられます。ペインの境界ドラッグによるサイズ変更、ペインの入れ替え、「表示」メニューからのプリセット切り替え、レイアウトの自動保存・復元ができます。

ペイン操作・プリセット・保存先などの手順の詳細は [docs/layout.md](docs/layout.md) を参照してください。

## VOICEVOX 連携（Phase 8）

「ツール → VOICEVOX セリフ入力」でダイアログを開くと、1 行に 1 セリフで入力した文面を VOICEVOX で音声合成し、再生ヘッド位置から順に**音声オブジェクト**と**同じ文面の字幕（テキストオブジェクト）**をタイムラインへ配置します。

- **VOICEVOX 本体は同梱していません。** 利用者が別途インストールし、起動しておく必要があります（動作確認: VOICEVOX 0.25.2 / `http://127.0.0.1:50021`）
- 話者選択（API から取得した話者一覧）、話速 / 音高 / 抑揚 / 音量、字幕の有無とフォントサイズ、配置レイヤー（音声 / 字幕）を指定できます
- 生成した WAV はプロジェクト保存先配下の音声フォルダに保存され、プロジェクトにはその参照（絶対パス）が記録されます。音声ファイルやプロジェクトの移動・削除後の読み込み時の追従（再リンク）は未確認です
- VOICEVOX 未起動の場合はエラーを表示し、編集は続行できます。「再試行」ボタンで再接続します
- 生成中はプログレスを表示し、キャンセルできます

### クレジット表記について（必読）

VOICEVOX および各音声ライブラリの利用規約に従い、**VOICEVOX で生成した音声を含む動画では、使用した音声ライブラリのクレジット表記が必要です**。話者・音声ライブラリごとの規約（クレジット表記の方法・必須可否）は **利用者本人の責任** で確認してください。MacUtl は VOICEVOX のクライアントであり、音声ライブラリの権利者ではありません。

VOICEVOX: <https://voicevox.hiroshiba.jp/>

## 動作環境

| 項目 | 要件 |
|---|---|
| CPU | Apple Silicon（arm64）。Intel Mac / Universal2 は配布しません |
| OS | macOS 27.0 以降（実測値。同梱 FFmpeg 9.0.2 の dylib が `minos 27.0` のため） |
| 空き容量 | 約 100 MB（展開後） |

macOS 27.0 未満では、同梱 dylib のロードに失敗する可能性があります。Intel Mac では動作しません。

## インストール

1. [Releases](https://github.com/Ren9618/MacUtl/releases) から `MacUtl-0.8.11-macos.dmg` をダウンロードします。
2. DMG を開き、`MacUtl.app` を `Applications` フォルダへドラッグします。
3. `/Applications/MacUtl.app` を起動します。

### 初回起動（Gatekeeper）

本アプリは **ad-hoc 署名のみ** で、Apple Developer ID 署名・公証（notarization）は行っていません。そのため、初回起動時に Gatekeeper が「開発元を確認できません」と表示します。以下のいずれかで起動してください。

- Finder で `MacUtl.app` を **右クリック →「開く」** を選び、ダイアログで「開く」を押す（推奨）
- 一度起動を試したあと、**システム設定 → プライバシーとセキュリティ → セキュリティ** の「このまま開く」を押す
- ターミナルで隔離属性を外す:

  ```bash
  xattr -dr com.apple.quarantine /Applications/MacUtl.app
  ```

Gatekeeper をシステム全体で無効化する操作（`spctl --master-disable` 等）は推奨しません。ZIP 版を使う場合も同様の手順が必要です。

## 既知の制限

- ベースの NeoUtl が **プレアルファ** です。API・プロジェクト形式・UI は予告なく変わります。
- **Intel Mac 非対応**（arm64 thin のみ）。
- VST3 等の音声プラグインは **スキャン確認のみ** で、実際の音声処理への適用は未確認です。
- **自動更新はありません。** 新しい版は Releases から手動で取得してください。macOS では `.app` の自己置換がコード署名を壊すため、自動更新を無効化しています。
- Apple Developer ID 署名・公証が無いため、Gatekeeper の警告が出ます。
- 起動時にシステムフォントを一括プリロードするため、一時的にメモリ使用量が増えることがあります。
- 本家 AviUtl のプラグインとの互換性はありません。
- VOICEVOX 本体は同梱していません。連携には利用者が別途 VOICEVOX をインストール・起動する必要があります。
- VOICEVOX の接続先 URL の変更は、現バージョンでは設定ファイルからのみ行えます（ダイアログの「接続先」入力欄が表示されない不具合があります。`voicevox.json` の `endpoint` を編集し、アプリ再起動後に反映されます）。
- 配置済みセリフの再合成（話者・文面を変えた差し替え）は未対応です。変更する場合は既存の音声・字幕オブジェクトを削除して再配置してください。
- 書き出しが完了した後、数分以内にアプリが予告なく終了する場合があります（確認済み）。書き出しファイル自体は正常に完成します。書き出し後はプロジェクトを保存してから作業を続けてください。原因は調査中です。
- 画像を含むプロジェクトを書き出すときは、先にプレビューを数秒再生してから書き出ししてください。画像テクスチャの初回読み込みが書き出しと重なると、書き出しが停止する場合があります。

## プライバシ

MacUtl は通常の編集・書き出し操作で外部サーバへデータを送信しません。

- クラッシュレポート（Sentry / GlitchTip）は **macOS では初期化されません**。上流の DSN 文字列はソースに残っていますが、macOS ビルドではコードレベルで無効化しています。
- 更新チェックは **macOS では無効** です。起動時に Codeberg の上流 API へ問い合わせることはありません。
- テレメトリ、利用統計、広告 SDK はありません。
- VST3 プラグインのスキャンはローカルのファイルシステムに対してのみ行います。
- VOICEVOX 連携時は、ローカルで起動した VOICEVOX エンジンの HTTP API（既定 `127.0.0.1:50021`）へ入力したセリフを送信します。インターネット上の外部サーバには送信しません。
- ユーザーが README 内の外部リンクを開いた場合のみ、ブラウザがそのサイトへ接続します。

## ライセンス

MacUtl は **GNU Affero General Public License version 3 or later（AGPL-3.0-or-later）** で配布します。上流 NeoUtl の `LICENSE` と著作権表示を保持しています。

- [LICENSE](LICENSE) — AGPL-3.0 全文
- [LICENSE_AUDIT.md](LICENSE_AUDIT.md) — ライセンス本調査（Rust / FFmpeg / branding / フォント / AGPL チェックリスト）
- [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) — 全 Rust クレートと同梱ネイティブライブラリのライセンス
- [MACOS_CHANGES.md](MACOS_CHANGES.md) — 上流からの変更点と変更日
- [third_party/](third_party/) — FFmpeg 等のライセンス本文と対応ソース情報

本アプリには GPL-3.0-or-later の FFmpeg（Homebrew 9.0.2、`--enable-gpl --enable-version3`）と、GPL の x264 / x265 が同梱されています。GPLv3 §13 に基づき AGPLv3 と組み合わせて配布しています。完全なソースはこのリポジトリで公開しています。

これまでの macOS 移植の作業報告（Phase 報告書）とリリースノートは [docs/macos/README.md](docs/macos/README.md) に一覧しています。

### 上流へのクレジット

- NeoUtl: <https://codeberg.org/taisho-guy/NeoUtl>（AGPL-3.0-or-later）
- NeoUtl 公式サイト: <https://neoutl.taisho-guy.org>
- 上流 README: [README.upstream.md](README.upstream.md)
- 上流の履歴は書き換えず、`0ead393` の上にフォーク側の変更を積んでいます。

## 作者の他のソフト

このフォークの作者（レイノワークス）は、Windows 向けの無料解凍ソフト **Hodozip** や、ディスク使用量ビューアー **Yohaku** も作っています： https://ren9618.github.io/

## サポート

- サポートは **GitHub Issues のみ** です: <https://github.com/Ren9618/MacUtl/issues>
- 非同期対応で、返信・修正・対応時期を保証しません。
- 上流 NeoUtl / Codeberg へ、このフォークの不具合を報告しないでください。
- 不具合報告には、macOS のバージョン、Mac の機種、再現手順、可能ならログを添付してください。

## ソースからビルド

### 必要なもの

- macOS 27.0 以降 / Apple Silicon
- Xcode Command Line Tools
- Rust 1.99.1 以降（`rustup` 推奨）
- Python 3
- Homebrew の FFmpeg 9.0.2（GPL ビルド。`--enable-gpl --enable-version3 --enable-libx264 --enable-libx265` を含むもの）
- `git`、`hdiutil`、`codesign`（macOS 標準）

### 手順

```bash
git clone --recurse-submodules https://github.com/Ren9618/MacUtl.git
cd MacUtl

# 依存関係を取得してビルド
cargo xtask build --release

# MacUtl.app / DMG / ZIP を生成
cargo xtask package --release
```

生成物:

- `target/bundle/MacUtl.app`
- `target/bundle/MacUtl-0.8.11-macos.dmg`
- `target/bundle/MacUtl-0.8.11-macos.zip`

サブモジュール（`assets` / `i18n` / `sdk` / `wiki`）は参照のまま維持しています。`--recurse-submodules` を忘れた場合は `git submodule update --init --recursive` を実行してください。

上流 `i18n` サブモジュールの固定コミットには、macOS 向けの日本語診断文言が一部含まれていません。リリース版と同一の表示でビルドする場合は、次のパッチを適用してください（必須ではありません。未適用の場合はキー文字列がそのまま表示されます）。

```bash
git -C i18n apply ../patches/i18n-macutl-ja.patch
```

### テスト

```bash
cargo fmt --all -- --check
cargo test --workspace --exclude NeoUtl
```

## 謝辞

- NeoUtl 開発チーム（taisho-guy 氏ほか）
- AviUtl 作者の KEN 氏、および ExEdit の作者の方々
- VOICEVOX（ヒホ氏）および各音声ライブラリの権利者の皆様
- 依存クレート・FFmpeg・各コーデックの開発者

本プロジェクトは AviUtl からソースコードを流用したものではなく、NeoUtl のフォークとしてその成果物を利用しています。
