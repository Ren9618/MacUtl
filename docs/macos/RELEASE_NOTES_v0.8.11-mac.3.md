# MacUtl 0.8.11 (macOS fork Phase 8: VOICEVOX 連携)

NeoUtl v0.8.11 をベースにした、macOS ネイティブの AviUtl 風動画編集ソフト
**MacUtl** の Phase 8 リリースです。本家 AviUtl および NeoUtl の公式版・
移植版ではありません（非公式フォーク）。

## 収録内容

- Apple Silicon（arm64）向け `MacUtl.app`
- DMG: `MacUtl-0.8.11-macos.dmg`
- ZIP: `MacUtl-0.8.11-macos.zip`

### Phase 8 の変更

- **VOICEVOX 連携**: 「ツール → VOICEVOX セリフ入力」でダイアログを開き、
  1 行に 1 セリフで入力した文面をローカルの VOICEVOX エンジンで音声合成し、
  **音声オブジェクトと同じ文面の字幕（テキストオブジェクト）** を再生ヘッド位置から
  順にタイムラインへ配置します。
  - 話者はエンジンから取得した一覧から選択します（動作確認: VOICEVOX 0.25.2 / 話者 127 件）
  - 話速・音高・抑揚・音量のスライダー、字幕の有無とフォントサイズ、
    音声 / 字幕の配置レイヤーを指定できます
  - 生成中はプログレスを表示し、キャンセルできます
  - VOICEVOX 本体は同梱していません。利用者が別途インストールし、起動しておく必要があります
- **音声付き書き出し**: 音声オブジェクトを含むプロジェクトの書き出し MP4 に
  AAC 音声ストリームを含めます（音声のないプロジェクトは従来どおり映像のみ）。

### クレジット表記について

VOICEVOX および各音声ライブラリの利用規約に従い、**VOICEVOX で生成した音声を
含む動画では、使用した音声ライブラリのクレジット表記が必要です**。
話者・音声ライブラリごとの規約（表記の方法・必須可否）は **利用者本人の責任** で
確認してください。詳細は
[README の VOICEVOX 節](https://github.com/Ren9618/MacUtl#voicevox-連携phase-8)
を参照してください。

### 回帰確認

- `cargo fmt --all -- --check`: 合格
- テスト: `NeoUtl` 71/71 パス、workspace 80 バイナリすべてパス（0 失敗）
- リリースビルド（`cargo xtask build --release`）: 成功
- パッケージ版（DMG 収録の `MacUtl.app`）でのスモーク確認:
  - 起動して一画面レイアウト（プレビュー＋プロパティ＋タイムライン）が表示される
  - 「ツール → VOICEVOX セリフ入力」でエンジン 0.25.2 / 話者 127 件に接続
  - phase2 動画＋画像＋音声プロジェクトの 720p 書き出しが完走
    - h264 1280×720 / 391 フレーム / 13.000 秒 ＋ AAC 48 kHz 2ch / 13.033 秒
    - SHA-256 `4ff5d20509e7d950eda58d9f73bd6d32a9afe97caa77b5f49d60fd90cf4a1aeb`
    - Phase 8 検証の基準出力（`kb/s:44.85`）と **ビット単位で一致**

## インストール

1. `MacUtl-0.8.11-macos.dmg` をダウンロードします。
2. DMG を開き、`MacUtl.app` を `Applications` へドラッグします。
3. 初回起動は Finder で `MacUtl.app` を **右クリック →「開く」** を選んでください。
   ad-hoc 署名のみのため、Gatekeeper の警告が出ます。
   詳しい手順は [README](https://github.com/Ren9618/MacUtl#初回起動gatekeeper) を参照してください。

ZIP 版を使う場合も同じ手順です。

## 動作環境

- Apple Silicon（arm64）のみ。Intel Mac は非対応です。
- macOS 27.0 以降（同梱 FFmpeg 9.0.2 の dylib が `minos 27.0` のため）

## 既知の制限

- 書き出しが完了した後、数分以内にアプリが予告なく終了する場合があります（確認済み）。
  書き出しファイル自体は正常に完成します。書き出し後はプロジェクトを保存してから
  作業を続けてください。原因は調査中です。
- 画像を含むプロジェクトを書き出すときは、先にプレビューを数秒再生してから
  書き出ししてください。画像テクスチャの初回読み込みが書き出しと重なると、
  書き出しが停止する場合があります。
- VOICEVOX ダイアログの「接続先」入力欄が表示されません（不具合）。
  接続先の変更は設定ファイル（`voicevox.json` の `endpoint`）から行ってください。
- 配置済みセリフの再生成（話者・文面の差し替え）の導線はありません。
  既存の音声・字幕オブジェクトを削除して再配置してください。
- 生成した WAV への参照はプロジェクトに絶対パスで記録されます。
  プロジェクトや WAV の移動後の追従（再リンク）は未対応です。
- VOICEVOX 本体は同梱していません。連携には別途インストール・起動が必要です。
- VST3 等の音声プラグインはスキャン確認のみで、実適用は未確認です。
- 自動更新はありません。新版は Releases から手動で取得してください。
- Apple Developer ID 署名・公証は行っていません。Gatekeeper の警告が出ます。
- ベースの NeoUtl はプレアルファです。API・プロジェクト形式・UI は変わることがあります。
- 本家 AviUtl のプラグインとのバイナリ互換はありません。

## プライバシ

通常の編集・書き出し操作で外部サーバへデータを送信しません。
VOICEVOX 連携時は、利用者がローカルで起動した VOICEVOX エンジンの HTTP API
（既定 `127.0.0.1:50021`）へ入力したセリフを送信します。インターネット上の
外部サーバには送信しません。macOS ではクラッシュレポート（Sentry / GlitchTip）と
更新チェックをコードレベルで無効化しています。テレメトリや広告 SDK はありません。

## ライセンス

AGPL-3.0-or-later。上流 NeoUtl の LICENSE と著作権表示を保持しています。
FFmpeg（GPL-3.0-or-later、Homebrew 9.0.2）と x264 / x265 を同梱しています。
VOICEVOX 本体および音声ライブラリは同梱していません。
GPLv3 §13 に基づき AGPLv3 と組み合わせて配布します。

- [LICENSE](https://github.com/Ren9618/MacUtl/blob/main/LICENSE)
- [LICENSE_AUDIT.md](https://github.com/Ren9618/MacUtl/blob/main/LICENSE_AUDIT.md)
- [THIRD_PARTY_NOTICES.md](https://github.com/Ren9618/MacUtl/blob/main/THIRD_PARTY_NOTICES.md)
- [MACOS_CHANGES.md](https://github.com/Ren9618/MacUtl/blob/main/MACOS_CHANGES.md)
- 上流 NeoUtl: <https://codeberg.org/taisho-guy/NeoUtl>
- VOICEVOX: <https://voicevox.hiroshiba.jp/>

## サポート

GitHub Issues のみ・非同期対応です。返信や修正を保証するものではありません。
<https://github.com/Ren9618/MacUtl/issues>

## チェックサム

```text
DMG SHA-256: 1cdaaeb68f901a73d1285c407350fcc4a10e65e8bd95c42120c782973b14a62e
ZIP SHA-256: 726580d6654d56c3eee19756b8e31fe131b422494d56ea73083e9263e1b0c071
```
