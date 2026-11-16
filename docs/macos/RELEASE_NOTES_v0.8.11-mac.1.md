# MacUtl 0.8.11 (macOS fork 初回リリース)

NeoUtl v0.8.11 をベースにした、macOS ネイティブの AviUtl 風動画編集ソフト
**MacUtl** の初回リリースです。本家 AviUtl および NeoUtl の公式版・
移植版ではありません（非公式フォーク）。

## 収録内容

- Apple Silicon（arm64）向け `MacUtl.app`
- DMG: `MacUtl-0.8.11-macos.dmg`
- ZIP: `MacUtl-0.8.11-macos.zip`

### 主な機能

- タイムライン編集、Undo / Redo、プロジェクトの保存・読み込み
- VideoToolbox による HW デコード（失敗時はソフトウェアへフォールバック）
- 日本語表示と IME による直接入力
- Objects 9 種 / Effects 23 種 / Decoders 3 種 / Easing 1 種
- H.264 MP4 の 720p / 1080p 書き出し（動画＋画像プロジェクトを含む）

### このリリースでの修正

- 動画＋画像オブジェクトを同時に使った書き出しが停止する問題
  （wgpu / parking_lot の lost wakeup）を macOS 向けの共有ロックで修正
- macOS の自動更新と上流クラッシュレポート送信を無効化

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

- ベースの NeoUtl はプレアルファです。API・プロジェクト形式・UI は変わることがあります。
- Intel Mac / Universal2 は配布していません。
- VST3 等の音声プラグインはスキャン確認のみで、実適用は未確認です。
- 自動更新はありません。新版は Releases から手動で取得してください。
- Apple Developer ID 署名・公証は行っていません。Gatekeeper の警告が出ます。
- 本家 AviUtl のプラグインとのバイナリ互換はありません。

## プライバシ

通常の編集・書き出し操作で外部サーバへデータを送信しません。
macOS ではクラッシュレポート（Sentry / GlitchTip）と更新チェックを
コードレベルで無効化しています。テレメトリや広告 SDK はありません。

## ライセンス

AGPL-3.0-or-later。上流 NeoUtl の LICENSE と著作権表示を保持しています。
FFmpeg（GPL-3.0-or-later、Homebrew 9.0.2）と x264 / x265 を同梱しています。
GPLv3 §13 に基づき AGPLv3 と組み合わせて配布します。

- [LICENSE](https://github.com/Ren9618/MacUtl/blob/main/LICENSE)
- [LICENSE_AUDIT.md](https://github.com/Ren9618/MacUtl/blob/main/LICENSE_AUDIT.md)
- [THIRD_PARTY_NOTICES.md](https://github.com/Ren9618/MacUtl/blob/main/THIRD_PARTY_NOTICES.md)
- [MACOS_CHANGES.md](https://github.com/Ren9618/MacUtl/blob/main/MACOS_CHANGES.md)
- 上流 NeoUtl: <https://codeberg.org/taisho-guy/NeoUtl>

## サポート

GitHub Issues のみ・非同期対応です。返信や修正を保証するものではありません。
<https://github.com/Ren9618/MacUtl/issues>

## チェックサム

```text
DMG SHA-256: b08ec088ccf18d33f59688add3d763aa2605a9653f363a28dcfc3b4e9aa05548
ZIP SHA-256: d1d689d03cf0f1a735d182fe9c27b8d1bd4f6b2e3fa5e406c716bc0619252aba
```
