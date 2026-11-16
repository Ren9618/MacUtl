# MacUtl 0.8.11 (macOS fork Phase 7: 一画面ワークスペース)

NeoUtl v0.8.11 をベースにした、macOS ネイティブの AviUtl 風動画編集ソフト
**MacUtl** の Phase 7 リリースです。本家 AviUtl および NeoUtl の公式版・
移植版ではありません（非公式フォーク）。

## 収録内容

- Apple Silicon（arm64）向け `MacUtl.app`
- DMG: `MacUtl-0.8.11-macos.dmg`
- ZIP: `MacUtl-0.8.11-macos.zip`

### Phase 7 の変更

- **一画面レイアウト**: プレビュー・プロパティ・タイムラインを 1 ウィンドウに統合
- **YMM4 風の既定配置**: 上段にプレビュー＋プロパティ、下段にタイムライン全幅
- **従来レイアウト**: プレビュー全幅＋下段 2 分割のプリセットを追加
- **サイズ変更**: ペイン境界のドラッグで縦・横ともリサイズ
- **ペイン入れ替え**: ヘッダのドラッグ、またはヘッダ右クリックの「◯◯と入れ替え」
- **レイアウト初期化**: ヘッダ右クリックと「表示」メニューから実行
- **自動保存・復元**: `~/Library/Application Support/MacUtl/settings/workspace-layout.json`
  （開発ビルドでは実行ファイル隣の `settings/workspace-layout.json`）
- **最小サイズ**: ウィンドウ 800×600。極小ペインでも破綻しないガードを追加
- 再生ツールバーをプレビュー下部へ移動
- 狭いエフェクト一覧では「有効」バッジを省略するか名前を切り詰めて、重なりを防止

### 回帰確認

- 720p 動画＋画像プロジェクトの書き出し: 391 フレーム / 1280×720 / 13.000 秒
  - SHA-256 `bccf52e6a5424a1feea36614ebc451ed77df0afd598f409099d8af807d2fab2b`
  - Phase 5 の基準出力と一致
- 保存／読込、再生・シーク、日本語表示、IME 直接入力を確認

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

- ペインをフローティングの別ウィンドウにはできません（v1 の対象外）。
- システム設定・プロジェクト設定・シーン設定・キーバインド・書き出し・
  エフェクト追加・イージングエディタは、従来どおり別ウィンドウで開きます。
- 960 幅ではプロパティのエフェクト一覧が狭く、「有効」バッジが省略されるか、長いエフェクト名が省略されることがあります。
- 書き出しは映像のみで、音声ストリームは含みません（既存仕様）。
- ベースの NeoUtl はプレアルファです。API・プロジェクト形式・UI は変わることがあります。
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
DMG SHA-256: 4510a61e5e69a4277ef8b3d605f739405c6c6ca6b8b23f4c9ef2b79b75b6de70
ZIP SHA-256: 83e53a23581cf8a0e46ca3c97401ab6101a5511906ef4081de7a605b16205ad3
```