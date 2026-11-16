# MacUtl 開発記録（macOS 移植の報告書とリリースノート）

このフォルダには、NeoUtl をベースにした macOS 移植（MacUtl）の、フェーズごとの作業報告書と公開済みリリースのリリースノートを置いています。導入手順などの全体情報はリポジトリルートの [README.md](../../README.md) を参照してください。

## Phase 報告書

| ファイル | 日付 | 内容 |
|---|---|---|
| [MACOS_PHASE0_1_REPORT.md](MACOS_PHASE0_1_REPORT.md) | 2026-11-12 | Phase 0（調査・ブロッカー棚卸し・ライセンス一次確認）と Phase 1（最小起動確認）の報告 |
| [MACOS_PHASE2_REPORT.md](MACOS_PHASE2_REPORT.md) | 2026-11-12 | 基本動作の E2E 検証（プロジェクト作成・プレビュー再生・編集操作・保存・書き出し・1080p スケール・安定性） |
| [MACOS_PHASE3_REPORT.md](MACOS_PHASE3_REPORT.md) | 2026-11-12 | macOS 向け残課題の本対応（IME 直接入力・VideoToolbox HW デコード・オブジェクト／エフェクト拡充・保存・書き出しの安定化など） |
| [MACOS_PHASE4_REPORT.md](MACOS_PHASE4_REPORT.md) | 2026-11-12 | `.app` 化・配布パッケージ（DMG / ZIP 生成・ad-hoc 署名・Gatekeeper 対応） |
| [MACOS_PHASE45_REPORT.md](MACOS_PHASE45_REPORT.md) | 2026-11-12 | 動画＋画像オブジェクトの書き出し停止の修正 |
| [MACOS_PHASE5_REPORT.md](MACOS_PHASE5_REPORT.md) | 2026-11-12 | MacUtl 公開・初回リリース（v0.8.11-mac.1） |
| [MACOS_PHASE7_REPORT.md](MACOS_PHASE7_REPORT.md) | 2026-11-13 | 一画面ワークスペース（YMM4 風レイアウト）の実装（v0.8.11-mac.2） |

## リリースノート

| ファイル | 内容 |
|---|---|
| [RELEASE_NOTES_v0.8.11-mac.1.md](RELEASE_NOTES_v0.8.11-mac.1.md) | v0.8.11-mac.1（2026-11-12 公開）のリリース本文 |
| [RELEASE_NOTES_v0.8.11-mac.2.md](RELEASE_NOTES_v0.8.11-mac.2.md) | v0.8.11-mac.2（2026-11-13 公開）のリリース本文 |

上流からの変更点と変更日の一覧はルートの [MACOS_CHANGES.md](../../MACOS_CHANGES.md) を参照してください。
