# MacUtl Phase 8 報告書（VOICEVOX 連携）

- 日付: 2026-11-17
- ベース: `main` = `b2e2647e`（Phase 7 = v0.8.11-mac.2 公開済み。仕様書記載の `2732e645` から docs コミット 2 件が後続した先頭。GitHub `origin/main` に対し ahead 10・未 push）
- 範囲: Phase 8（VOICEVOX 連携による声入れ）の仕上げ＋検証＋報告。**commit / push / リリースは未実施**（仕様どおり）
- 実機: Apple Silicon MacBook Air（画面 1680×1050 points）/ VOICEVOX 0.25.2 が `http://127.0.0.1:50021` で稼働

## 1. 着手時の作業ツリー状態

仕様書の「現状」記述と一致した:

- `src/voicevox/`（`client.rs` / `runner.rs` / `settings.rs` / `mod.rs`）と `src/ui/voice_dialog.rs` が未コミットで残存
- `mod.rs` がサブモジュール宣言のみで re-export が無く、`voice_dialog.rs` の import 5 項目が未解決（E0432 ×1、E0425 ×2 相当）
- `MACOS_PHASE7_INVESTIGATION.md`（untracked）は触らず保持
- 追加確認: 音声書き出し経路（`src/audio/export.rs` 新規・`src/audio/mixer.rs`・`src/project/export.rs`・`crates/media/ffmpeg/src/encoder.rs` + `lib.rs`）も前回からの未コミット差分として残っていた。仕様書の現状記述には明記がなかったが、前回セッションの Phase 8 作業の一部としてそのまま引き継いだ（書き換え・巻き戻しはしていない）

## 2. 修復内容（今回分）

1. `src/voicevox/mod.rs`: `runner.rs` 実装済みの `VoiceClip` / `VoiceGenerationRequest` / `split_utterances` / `spawn_speakers` / `spawn_generation` を `pub use runner::{…}` で re-export。実体の新規実装は不要だった（すべて runner.rs / client.rs に存在）
2. `src/voicevox/runner.rs`: テストの修復（`wav_duration_seconds`・`split_utterances`・`apply_query_parameters` の 5 テストが通る形）。`src/voicevox/` はディレクトリごと untracked のため差分はファイル全体で確認のこと
3. その他のソース修正なし。前回実装（voice_dialog.rs の設計・voicevox 各ファイル）は書き直していない

## 3. 棚卸し結果（計画書 §5 との突合）

「前回分」= 2026-11-13 夜の未コミット実装、「今回」= 本日の補完。新規オブジェクト種の追加・プロジェクト形式の変更は行っていない（`.npb` 上流互換維持）。

| 計画 §5 の項目 | 到達状況 | 実装の所在 |
|---|---|---|
| セリフ追加パネル（レイアウト 1 パネル or ダイアログフォールバック） | 前回分。**ダイアログ方式**（ツール →「VOICEVOX セリフ入力」）で適合 | `src/ui/voice_dialog.rs` |
| セリフ本文（複数行・改行ごとに 1 セリフ） | 前回分。`split_utterances`（空白行スキップ・前後空白トリム・行長上限 `MAX_LINE_CHARS` あり） | `src/voicevox/runner.rs` |
| 話者選択（API から一覧取得・取得できない場合はその旨表示） | 前回分。`/version` → `/speakers` でエンジン版＋話者一覧を取得。失敗時は赤字エラー＋「再試行」 | `src/voicevox/client.rs` / `voice_dialog.rs` |
| パラメータ（話速・音高・抑揚） | 前回分。UI スライダーは話速 0.5–2.0 / 音高 ±0.15 / 抑揚 0–2 / **音量 0–2**（計画 3 項目＋音量） | `voice_dialog.rs` → `apply_query_parameters` |
| 配置位置: 再生ヘッド位置・対象レイヤー | 前回分。音声レイヤー / 字幕レイヤーを個別指定（0–127）、再生ヘッドから順に配置 | `voice_dialog.rs` |
| WAV 生成 → 保存 → 音声オブジェクト配置（WAV 実尺に長さ合わせ） | 前回分。WAV は `<プロジェクト保存先>/voicevox/voice_<時刻>_<nn>.wav`、長さは `wav_duration_seconds` で実尺 | `runner.rs` |
| 字幕テキストオブジェクトの自動配置（有無・フォントサイズ切替） | 前回分。字幕チェックボックス＋フォントサイズ 12–120 スライダー | `voice_dialog.rs` / settings |
| 接続先 URL 設定可能 | **部分実装**。値は `target/debug/settings/voicevox.json`（`endpoint`）に永続化され、既定は `default_endpoint()`。ただしダイアログの「接続先」入力欄が表示されず UI からは変更できない（§5.6 参照）→ v2 修正候補 | `settings.rs` / `voice_dialog.rs` |
| 未起動・接続失敗時のエラー表示＋再試行、編集を止めない | 前回分。**本日実測**（§4.3） | `voice_dialog.rs` |
| 生成中のプログレス＋キャンセル | 前回分。進捗 `n / m`＋プログレスバー、「閉じる」ボタンが「キャンセル」に切替 | `voice_dialog.rs` / `runner.rs`（`AtomicBool` キャンセル） |
| 配置済みセリフの再生成（差し替え）導線 | **未実装**。計画の判断ゲートどおり「削除＋再追加」運用として **v2 候補に明記** | — |
| クレジット表記の README 明記 | **本日補完**。README に「VOICEVOX 連携（Phase 8）」セクション＋クレジット必須の注意＋話者ごとの規約は利用者責任、を追記 | `README.md` |

## 4. 検証結果

### 4.1 ビルド・テスト（実測）

- `cargo fmt --all -- --check`: 合格
- `cargo test -p NeoUtl`: **71/71 パス**
- `cargo test --workspace --exclude NeoUtl`: **80 バイナリ全パス**
- `cargo build -p NeoUtl`: 成功（`target/debug/NeoUtl`）
- エンコーダは HW 候補（nvenc / qsv / amf / vaapi）すべて未検出で libx264 ソフトウェアに縮退（Phase 7 と同じ挙動）

### 4.2 VOICEVOX 実機 E2E（実機確認。スタブ不使用）

VOICEVOX 0.25.2（本機で稼働中）に対し、実機で確認:

1. 「ツール → VOICEVOX セリフ入力」でダイアログが開く。「エンジン 0.25.2 / 話者 127 件」で自動接続
2. 話者: 四国めたん（ノーマル、style_id 2）。2 行（「Hello MacUtl.」/「Voice and subtitle test.」）を 1 行に 1 セリフで入力して「生成して配置」
   - ダイアログのテキストエリアでは IME が乱変換を起こすため、本検証は **pbcopy + 貼り付け（Cmd+V）** で入力（§5.7 の IME 課題）
3. WAV 2 件が `<プロジェクト>/voicevox/` に生成（24kHz mono 16bit PCM、1.312s / 1.995s）
4. タイムラインにレイヤー 1 = 音声 2 件、レイヤー 2 = 字幕テキスト 2 件が配置され、プレビューに「Hello MacUtl.」表示
5. 保存 → アプリ再起動 → 再オープンで欠落なし復元（`project.npb` に WAV の**絶対パス** 2 件＋セリフ 2 行を確認）
6. 720p 音声付き書き出し: 下表 1 行目のとおり完成。音量測定で発話区間を確認

書き出し実測（ffprobe / volumedetect / shasum）:

| ファイル | 映像 | 音声 | サイズ | 備考 |
|---|---|---|---|---|
| `target/phase8/export_voice_720p.mp4` | h264 1280×720 10.0s | aac 48000Hz 2ch 10.033s | 132,078 bytes | max −8.3 dB、発話区間 0–3.22s（2 行連続配置と一致） |
| `target/phase8/regress_phase2_720p.mp4` | h264 1280×720 13.000s 391 フレーム | aac 48kHz 2ch 13.033s | 163,483 bytes | x264 `kb/s:44.85` は Phase 7 報告値と完全一致。max −21.0 dB（440Hz トーン）。SHA-256 `4ff5d205…1aeb` |
| `target/phase8/noaudio_untitled.mp4` | h264 1280×720 10.0s 301 フレーム | **なし（ストリーム数 1）** | 18,529 bytes | 音声なしプロジェクトでは `has_active_audio` 分岐により音声ストリームを付けないことを確認 |

**回帰基準の扱い**: Phase 7 の phase2_e2e_720p は音声オブジェクト（`audio_440hz_10s.wav`）を含むため、Phase 8 では書き出しに音声トラックが付与されるのが正しく、Phase 7 の映像のみハッシュ（79,360 bytes / `bccf52e6…`）は設計上再現不能。そのため (a) 391 フレーム / 13.000 秒 / x264 `kb/s:44.85` の一致、(b) 音声なしプロジェクトでストリーム数 1、の 2 点で回帰を判定した（§4.2 表 2・3 行目）。

### 4.3 VOICEVOX 未接続時の挙動（実測）

`voicevox.json` の `endpoint` を死んだポート（`http://127.0.0.1:59999`）に変更 → アプリ再起動 → ダイアログ自動接続:

- 赤字エラー表示: 「VOICEVOX に接続できません（バージョン取得: io: Connection refused (os error 61)）。http://127.0.0.1:59999 で VOICEVOX が起動しているか確認してください。」
- 「再試行」ボタン表示、話者選択は「話者一覧を取得できません」で無効化
- ダイアログ以外の通常編集（タイムライン・プレビュー・保存）は妨げられない。クラッシュ・ハングなし
- エンドポイントを 50021 に戻して再起動 → 再び「エンジン 0.25.2 / 話者 127 件」で正常接続（回復 confirmed）

### 4.4 その他の回帰確認

- 既存プロジェクト（phase8_voice / phase2_e2e_720p / autosave された Untitled）の起動時復元: OK
- 一画面レイアウト復元: OK（プレビュー＋プロパティ＋タイムライン）
- IME 直接入力: タイムライン・ダイアログで「貼り付け」経由が安全という運用は前回報告どおり（§5.7）
- 音声なしプロジェクトの 720p 書き出し: §4.2 表 3 行目のとおり音声ストリーム不付与を確認

## 5. 今回発見した不具合（残課題）

### 5.1 書き出し完了後、数分でアプリが無音終了する（×2 回再現、原因未特定）

- 再現 1: 09:47 回帰書き出し完了 → 09:50–09:52 の間にプロセス消失（pid 87977）
- 再現 2: 09:58 音声なし書き出し完了 → 09:59:12 のダイアログ操作後〜10:00 頃に消失（pid 89501）
- 共通点: ログは x264 統計で正常終了し、**panic も crash レポートも無し**（終了コード不詳）
- 切り分け実験: 別アプリへのフォーカス移動（5 秒）では再現せず → 単純な非アクティブ化が直接の死因ではない
- 前回セッションで報告の「ネイティブパネル非アクティブ化でアプリ終了」とは条件が異なる可能性。書き出し完了後のタイマー系処理（autosave / レンダーキュー後始末）が疑われるが、本フェーズでは特定に至らず。**終了コード記録ラッパーを準備済み**（`/tmp/neoutl_run.sh`）で、次回検証時に確定できる

### 5.2 画像テクスチャ初回ロードと書き出しの並行で書き出しが停止する（既知障害の全容解明）

- 症状: 画像を含むプロジェクトで、書き出し中に画像テクスチャの初回ロード（dylib 経由）が発生すると、全スレッドが停止して書き出しが完了しない（今回フレーム 42 付近で 2 回発生）
- デッドロックチェーン（スレッドサンプルで確定）:

| スレッド | 保持中 | 待ち対象 |
|---|---|---|
| メイン（egui submit） | wgpu `command_indices` 書込 | wgpu `pending_writes`（unpark されず） |
| video-decoder（map_async 中） | `shared_wgpu_submit_lock` | wgpu `command_indices` |
| RenderQueue（export::run） | EcsWorld ＋ engine | `shared_wgpu_submit_lock` |
| autosave | — | EcsWorld |

- 根拠: `crates/media/runtime/src/cache.rs:556` のコメントが同一障害（画像デコーダ dylib が wgpu / parking_lot を別コピー静的リンク → 待機解除が別コピー間で成立しない）を明記。wgpu-core 30.0.1 の `Queue::submit` は `allocate_submission` で `command_indices` 書込を取得し submit 全体で保持するため、dylib 側 `write_texture` とメインスレッド submit が重なると該当
- Phase 8 の音声実装（audio/export.rs・mixer.rs・encoder.rs）は wgpu / 共有ロックに一切触れておらず、無関係（音声混合は停止前に正常完了）
- **確定的回避手順**: 書き出し前にプレビュー再生（約 2.5 秒）で PNG テクスチャをロードしておく。プレビュー経由ならメインスレッドがアップロードと自前 submit を逐次実行するため競合しない。本検証はこの手順で回帰書き出しを完遂（§4.2）
- 恒久対策は v2 候補（共有ロック緩和の穴: 画像テクスチャ初回アップロードとメインスレッド egui submit の直列化）

### 5.3 VOICEVOX ダイアログの「接続先」入力欄が表示されない

- コード上は `TextInput::new(&mut self.settings.endpoint)`（`voice_dialog.rs`）が存在するが、実測では「接続先」ラベルと「接続 / 話者更新」ボタンのみが描画され、入力欄が現れない（クリック・貼り付けも不反応）
- 影響: 接続先変更は設定ファイル（`voicevox.json`）経由のみ。README 既知の制限に記載済み。v2 修正候補

### 5.4 その他

- **再生成導線なし**: 配置済みセリフの話者・文面差し替えは未対応。「削除＋再追加」運用。v2 候補（計画の判断ゲートどおり）
- **WAV 参照は絶対パス**: `project.npb` に絶対パス記録。プロジェクト移動・WAV 削除後の再リンク導線は未実装（読込は正常）
- **VST3 音声プラグイン**: Phase 7 から変更なし。スキャン確認のみで実適用は v1 非対応
- **物理的な発音確認**: パイプライン（生成 → 配置 → プレビュー → 書き出し AAC）まで実測済み。実際に声として聞こえるかは**ユーザーの耳確認**として分離（Phase 2 以来の宿題と合わせて実施推奨）

## 6. ドキュメント変更（今回分）

- `README.md`: 「VOICEVOX 連携（Phase 8）」セクション新設（仕組み・前提・WAV 保存先・未接続時挙動）＋**クレジット表記必須の注意と話者ごとの規約は利用者責任**の明記（計画 §5.3・§6 対応）＋既知の制限 3 件追加＋プライバシにローカル API 送信の明記＋謝辞に VOICEVOX 追加＋「現在できること」を Phase 8 実測版に更新

## 7. 検証成果物と後片付け

- `target/phase8/`: 書き出し実測ファイル 3 件（§4.2 表）＋ `target/phase8_app.log`（検証ログ）は報告の証跡として保持
- 以下の一時ツール・スクリーンショットは検証終了後に除去した（本報告に記録のうえ、git には入らない `target/` 配下）: `mouse` / `key` ビルド＋ `.swift`（画面操作用 HID ツール）、`screenshot_*.png` 計 73 枚、`btn_area.png` / `endpoint_row.png` 等の解析用クロップ、`audio_check/`（音量解析用の WAV 抽出）
- VOICEVOX 本体の同梱・インストールは行っていない（実機の既存 0.25.2 を使用）。スタブサーバも不使用
- 上流（Codeberg）への push / PR / Issue なし。認証情報をファイルに含めていない

## 8. ユーザー側に残る判断・宿題

1. **リリース判断**: 本報告を踏まえた Phase 8 の GO/NO-GO。GO の場合 `v0.8.11-mac.3`（計画 §6 の想定）で commit / push / リリースを次プロンプトで指示
2. **物理的な発音確認**: 生成音声・書き出し MP4 の声が実際に聞こえるかの耳確認
3. **v2 候補の採否**: (a) 再生成（差し替え）導線、(b) 接続先入力欄の修復、(c) 書き出し後の無音終了バグ、(d) 画像 dylib デッドロックの恒久対策、(e) WAV 参照の相対化／再リンク
4. **IME**: ダイアログのテキストエリアで IME 乱変換が残存（貼り付け運用で回避可能）。修復は v2 候補
