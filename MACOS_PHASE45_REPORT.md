# macOS Phase 4.5 報告書 — 動画＋画像オブジェクト書き出し停止の修正

- **日付**: 2026-11-12
- **対象**: NeoUtl macOS / version 0.8.11
- **基準 HEAD**: `0ead39350db55996810de53da79d02fbc98355f5`（変更はすべて未コミット・ローカルのみ）
- **作業ディレクトリ**: `/Users/ray_9618/Desktop/Git/NeoUtl-macOS`
- **結論**: 動画＋画像を含むプロジェクトの書き出しがフレーム 43→44 で停止する問題を再現・最小化し、macOS 専用の最小修正で解消した。HW デコードは無効化していない。最終 `.app` と実バイナリの双方で 391 フレーム書き出しを確認し、1080p/60 秒は 37 秒（Phase 2: 36.90 秒）で回帰なし、クラッシュ 0 件。

---

## 1. 要約

| 項目 | 結果 |
|---|---|
| 再現 | 動画＋画像の同時配置で、書き出しが `progress 43` の直後に停止（ベアバイナリ・`.app` の双方） |
| 最小化 | 動画のみ / 画像のみ / 動画＋テキストは完走。動画＋画像のみ停止 |
| 原因 | 画像デコーダ dylib が wgpu / parking_lot を別コピーで静的リンクし、共有 Metal キューの `pending_writes` 上で待機解除が失われる（lost wakeup） |
| 分類 | 主因は **(c) 上流・共通設計**（プラグイン dylib ごとの wgpu/parking_lot 重複）。**(d) プロジェクト/メディア固有**の動画＋画像の同時使用が引き金。Phase 3 のデコーダ変更 (a) やバンドル環境 (b) 単独が原因ではない |
| 修正 | macOS のみ、画像 `texture()`・デコーダ `map_async`・`write_buffer` をホスト共有 wgpu ロックで直列化。HW デコードは維持 |
| 391 フレーム検証 | 最終 `.app` の通常 UI 書き出し、最終ベアバイナリの通常 UI 書き出し、および再構築 `.app` の連続 3 回（ハーネス使用）ですべて 391 フレーム完走 |
| 1080p/60 秒 | **37 秒**（Phase 2: 36.90 秒）、1801 フレーム、クラッシュ 0 |
| プレビュー | VideoToolbox HW 初期化成功を維持。初期再生ピーク RSS **89.19 MB**（Phase 3: 138.8 MB） |
| 品質確認 | `cargo fmt --all -- --check` 合格、`cargo test --workspace --exclude NeoUtl` は 80 件の `test result: ok`、failed 0 |
| 制約 | コミット・push・アップロードなし。Apple ID / 証明書 / notarization / sudo 未使用。Stage Manager は `GloballyEnabled=1` に復元済み |

---

## 2. 再現と最小化

### 2.1 再現手順

1. `phase2_e2e_720p`（動画 `video_a_720p_10s.mp4`、画像 `image_800x600.png`、音声 `audio_440hz_10s.wav`）を開く。
2. 画像オブジェクトを読み込ませた状態で、`0..=334`（または `0..=390`）の書き出しを開始する。
3. `progress 43` を最後に進捗が止まり、CPU 使用率が 0% になる。出力 MP4 は 48 bytes のまま。

再現用の一時ハーネス（環境変数 `NEOUTL_PHASE45_*`）で UI 操作なしに同じ書き出しを実行し、再現性を確認した。ハーネスは検証完了後にソースから完全に除去済み。

### 2.2 条件表（最小化）

| 条件 | 使用オブジェクト | 書き出し範囲 | 結果 |
|---|---|---:|---|
| 動画のみ | video | `0..=299`（300 フレーム） | **完走**（`only_video.log`） |
| 画像のみ | image | `0..=300`（301 フレーム） | **完走**（`only_image.log`） |
| 動画＋テキスト | video + text | `0..=390`（391 フレーム） | **完走**、約 60 秒（`video_text.log`） |
| 動画＋画像（HW デコード） | video + image | `0..=334` | **43→44 で停止**（`harness_repro.log`） |
| 動画＋画像（SW デコード） | video + image | `0..=334` | **43→44 で停止**（`harness_sw.log`） |
| 動画＋画像（`.app`） | video + image | `0..=390` | **43→44 で停止**（`app_stall_sample.log`） |
| 動画＋画像（修正後） | video + image | `0..=390` | **391 フレーム完走**（`fix_video_image.log`） |

- HW / SW の両方で停止するため、VideoToolbox 固有の問題ではない。
- 動画のみ・画像のみ・動画＋テキストが完走するため、単体メディアやテキスト合成の問題ではない。
- 停止は「動画と画像が同時に存在し、両方の GPU アップロードが重なる」場合に限られる。

### 2.3 停止時のスタック

停止時に `sample` で採取したスタック（`target/phase45/logs/app_stall.sample.txt`）の要点:

```text
書き出しスレッド (Thread_10684855)
  ui::start_project
  project::export::run                        export.rs:332
    RenderEngine::render                      frame.rs:26
      RenderEngine::render_at                 frame.rs:449
        infra::gpu_shared::locked_submit      gpu_shared.rs:49
          wgpu::Queue::submit
            wgpu_core::Queue::allocate_submission   queue.rs:1600
              parking_lot::RawRwLock::lock_exclusive_slow  raw_rwlock.rs:634
                __psynch_cvwait

デコーダスレッド (Thread_10684856 "neoutl-video-decoder")
  decoder::worker::decode_task
    decoder::worker::dispatch_frame           worker.rs:174
      frame_convert::compose_output_frame     frame_convert.rs:192
        colorconv::composite_p0xx_to_rgba     colorconv.rs:231
          colorconv::upload_plane_texture     colorconv.rs:176
            StagingPool::upload_plane         staging_pool.rs:116
              wgpu::Buffer::map_async
                wgpu_core::Queue::flush_writes_for_buffer  queue.rs:1296
                  parking_lot::RawMutex::lock_slow  raw_mutex.rs:262
                    __psynch_cvwait
```

- 両スレッドとも `queue.rs` 内の `pending_writes` 系ロックで `__psynch_cvwait` している。
- `libneoutl_media_image_decoder.dylib` はサンプル内のロード済みイメージ一覧にのみ存在し、**ロックを保持したまま実行中のスレッドは存在しない**。
- したがって「ロック保持による永久ブロック」ではなく、別コピーの parking_lot 間で wakeup が失われた **lost wakeup** である。
- 停止後しばらく放置すると `wgpu error: Validation Error ... In Surface::configure / Failed to wait for GPU to go idle` で abort する場合があるが、これは停止の結果であり原因ではない。

**ログ抜粋（`app_stall_sample.log`）:**

```text
[phase45] progress 43/390
[media-cache] 新規load: .../image_800x600.png
（以降 progress が増えない）
```

---

## 3. 原因分類

### 3.1 分類の評価

| 候補 | 判定 | 根拠 |
|---|---|---|
| (a) Phase 3 デコーダ変更 | **否** | SW デコード強制でも同一箇所で停止（`harness_sw.log`）。VideoToolbox 初期化の成否に依存しない |
| (b) バンドル環境 | **否（単独では）** | ベアバイナリ `target/release/NeoUtl` でも同一の 43/334 停止を再現（`harness_repro.log`）。`.app` 固有ではない |
| (c) 上流・共通設計 | **主因** | 画像デコーダが dylib 内に wgpu / parking_lot の別コピーを静的リンクし、同一 Metal キューの `pending_writes` を別ランタイムで待機する |
| (d) プロジェクト/メディア固有 | **引き金** | 動画＋画像の同時配置でのみ発生。動画のみ・画像のみ・動画＋テキストでは発生しない |

### 3.2 メカニズム

1. 画像デコーダはプラグイン dylib（`libneoutl_media_image_decoder.dylib`）としてロードされる。
2. その dylib はホストと同じ wgpu / parking_lot を**別コピー**として静的リンクしている。
3. 画像の `queue.write_texture`（dylib 内の wgpu）と、ホスト側の `queue.submit` / `map_async`（ホスト内の wgpu）が同じ `pending_writes` を触る。
4. `pending_writes` の待機・通知は parking_lot のコピーごとに独立しているため、一方のコピーが起こした wakeup が他方のコピーの待機者に届かず、両者が永久に待機する。
5. 特にデコーダの `map_async` → `flush_writes_for_buffer` と、書き出し側の `submit` → `allocate_submission` が同時に走るタイミングで顕在化する。

Phase 3 で導入済みの `submit` 同士の直列化だけでは、`map_async` / `write_texture` の経路が残るため不十分であった。

---

## 4. 修正

### 4.1 方針

- 共有 wgpu キューの `pending_writes` を触る操作を、ホストとデコーダで同一の共有ロックにより直列化する。
- 修正は **macOS のみ**（`#[cfg(target_os = "macos")]`）。他プラットフォームの挙動は変えない。
- HW デコードは無効化しない。
- Phase 3 で導入済みの `submit` ロックは維持する。

### 4.2 変更点

#### `crates/media/runtime/src/cache.rs`

画像テクスチャ生成（`image.decoder.texture(device, queue)`）を共有ロックで囲む。画像 dylib の `write_texture` がホストの `submit` / `map_async` と重ならないようにする。

```rust
#[cfg(target_os = "macos")]
let submit_lock = neo_media_ffmpeg::shared_wgpu_submit_lock();
#[cfg(target_os = "macos")]
let _submit_guard = submit_lock
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
image.texture = Some(image.decoder.texture(device, queue));
```

#### `crates/media/ffmpeg/src/decoder/mod.rs`

共有ロックを取得するヘルパーを追加。

```rust
pub fn lock_shared_wgpu_submit() -> std::sync::MutexGuard<'static, ()> {
    let lock = SHARED_QUEUE_SUBMIT_LOCK.get_or_init(|| Arc::new(Mutex::new(())));
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
```

#### `crates/media/ffmpeg/src/decoder/staging_pool.rs`

`map_async` を macOS のみ共有ロックで囲む。

```rust
{
    #[cfg(target_os = "macos")]
    let _gpu_lock = crate::decoder::lock_shared_wgpu_submit();
    slice.map_async(wgpu::MapMode::Write, move |result| {
        let _ = tx.send(result);
    });
}
```

#### `crates/media/ffmpeg/src/colorconv.rs`

`write_buffer` を macOS のみ共有ロックで囲む。

```rust
{
    #[cfg(target_os = "macos")]
    let _gpu_lock = crate::decoder::lock_shared_wgpu_submit();
    queue.write_buffer(&res.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
}
```

### 4.3 共通コードへの影響

- 新規の `lock_shared_wgpu_submit()` は共通コードに追加したが、**新しい呼び出しはすべて `cfg(target_os = "macos")` の内側**にあり、macOS 以外では実行されない。
- `cache.rs` の画像アップロード直列化、`staging_pool.rs` の `map_async` 直列化、`colorconv.rs` の `write_buffer` 直列化はいずれも macOS 専用。
- Phase 3 から存在する `submit` ロック（`gpu_shared::locked_submit` とデコーダ側 `submit`）は全プラットフォームで有効なまま。今回新たに一般コードの挙動を変えるものではない。
- `submit` ロックはネスト取得しないため、ロック順序逆転による新たなデッドロックはない。

### 4.4 HW デコードを無効化しない根拠

- 停止は SW デコード強制でも再現したため、HW デコードは原因ではない。
- 修正後も `VideoToolbox HW初期化成功 pix_fmt=AV_PIX_FMT_VIDEOTOOLBOX` を確認。
- 1080p/60 秒の書き出しは **37 秒**で、Phase 2 のソフトウェアデコード基準 **36.90 秒**と同等。HW デコードを無効化する測定上の根拠はない。

---

## 5. 検証結果

### 5.1 391 フレーム書き出し（最終ソース・ハーネスなし）

`phase2_e2e_720p` の画像終了フレームを 390 にしたプロジェクトを使用し、通常の UI（ランチャー → プロジェクト → File → メディアの書き出し → 書き出し）で実行。

| 実行環境 | 出力 | フレーム数 | 尺 | 結果 |
|---|---|---:|---:|---|
| 最終 `.app`（`target/bundle/NeoUtl.app`） | `target/phase45/exports/final_app_ui_391.mp4` | **391** | 13.000 秒 | 完走、クラッシュ 0 |
| 最終ベアバイナリ（`target/release/NeoUtl`） | `target/phase45/exports/final_bare_ui.mp4` | **391** | 13.000 秒 | 完走、クラッシュ 0 |

- 両出力は SHA-256 が一致: `bccf52e6a5424a1feea36614ebc451ed77df0afd598f409099d8af807d2fab2b`
- ファイルサイズ: 79,360 bytes（Phase 3 の 391 フレーム出力と同一サイズ）
- ffprobe: `1280x720`, `h264`, `nb_frames=391`, `nb_read_packets=391`, `duration=13.000000`
- 画像読み込みログ: `[media-cache] 新規load: .../image_800x600.png`（`.app` は 202 行目、ベアは 203 行目）
- 書き出しダイアログの終了フレーム 390 の記録: `target/phase45/screenshots/final_app_export_dialog_391.png`

### 5.2 再構築 `.app` の連続 3 回（391 フレーム）

修正後に再パッケージした `.app` で、一時ハーネスにより `0..=390` を 3 回連続実行。

| 実行 | 所要時間 | 進捗行数 | 出力サイズ | 結果 |
|---|---:|---:|---:|---|
| 1 回目 | 22 秒 | 391 | 49,603 bytes | `export OK` |
| 2 回目 | 20 秒 | 391 | 49,603 bytes | `export OK` |
| 3 回目 | 19 秒 | 391 | 49,603 bytes | `export OK` |

- 3 出力の SHA-256 はすべて一致: `1cc70e0b1b8545e5410d3803ba13130c8d15d60c3bbd4e4945f012c3f5ed71cf`
- ログ: `target/phase45/logs/app_run1.log` 〜 `app_run3.log`
- クラッシュ 0 件（`panicked` / `non-unwinding` / `abort` なし）

### 5.3 1080p / 60 秒ベンチマーク

| 項目 | Phase 2（SW デコード） | Phase 4.5（VideoToolbox HW） |
|---|---:|---:|
| 書き出し時間 | 36.90 秒 | **37 秒** |
| フレーム数 | 1801 | **1801** |
| 解像度 / 尺 | 1920×1080 / 60.0 秒 | 1920×1080 / 60.0 秒 |
| 出力サイズ | — | 59,877,522 bytes |

- `nb_frames=1801`, `nb_read_packets=1801`, `duration=60.000000`
- ログ: `target/phase45/logs/1080p_final.log`
- 実時間比は約 0.617 で、Phase 2 の 0.615 と同等。HW デコード継続による回帰はない。

### 5.4 プレビュー / HW デコード回帰

| 指標 | Phase 3 基準 | Phase 4.5 実測 | 判定 |
|---|---:|---:|---|
| VideoToolbox 初期化 | 成功 | **成功** | 維持 |
| 720p 初期再生ピーク RSS（frame 8〜13 付近） | 138.80 MB（142,131 KB） | **89.19 MB**（91,328 KB） | 改善（基準以下） |
| 720p 全編再生ピーク RSS | 未計測（Phase 3 ログは frame 8 で終了） | 301.47 MB（308,704 KB） | 参考値。RAM フォールバック容量 64 フレームによる別要因 |
| フォント初期化後のベース RSS | — | 約 80.7 MB | — |
| クラッシュ | — | 0 件 | 合格 |

- 初期再生は `target/phase45/logs/720p_frame8_samples.txt`、全編再生は `720p_preview_final_samples.txt`。
- 全編再生の 301.47 MB は、VRAM/RAM 予算取得失敗時の `RAM_FALLBACK_CAPACITY_NO_BUDGET=64`（1280×720 RGBA16F）によるフレームキャッシュが支配的で、今回のロック修正とは無関係。Phase 3 の 138.8 MB は再生が frame 8 で止まった時点の値であり、全編ピークとの単純比較はできない。
- VideoToolbox の HW フレーム転送経路は維持され、再生中の HW 初期化成功ログを確認。

### 5.5 クラッシュ・安定性

- 最終 `.app` UI 書き出し（391 フレーム）: クラッシュ 0
- 最終ベアバイナリ UI 書き出し（391 フレーム）: クラッシュ 0
- 再構築 `.app` 連続 3 回: クラッシュ 0
- 1080p/60 秒: クラッシュ 0
- プレビュー再生: クラッシュ 0
- 停止後の `wgpu Validation Error` は、修正前の停止状態でのみ発生。修正後の上記実行では発生していない。

### 5.6 コード品質

| コマンド | 結果 |
|---|---|
| `cargo fmt --all -- --check` | 合格（差分 0） |
| `cargo test --workspace --exclude NeoUtl` | 80 件の `test result: ok`、failed 0、ignored 7（doctest） |

- ログ: `target/phase45/logs/cargo_test_final2.log`
- 一時ハーネス（`PHASE45-TEMP-HARNESS` / `NEOUTL_PHASE45_*`）は `src/ui/mod.rs`、`src/app/mod.rs` から除去済み。一時診断用 example も削除済み。
- `src/ui/mod.rs` と `src/app/mod.rs` は HEAD と同一に戻っている。

### 5.7 391 フレームと 335 フレームの違い

- 書き出しダイアログの終了フレーム上限は `world.total_frames()`（`src/ui/export_dialog.rs`）。
- Application Support 側の既存 `phase2_e2e_720p` コピーは画像終了フレームが 334 のため、通常 UI では **335 フレーム（0..=334）** になる。これは仕様どおりのダイアログ上限であり、停止ではない。
- 391 フレーム検証では、画像終了フレーム 390 のプロジェクトコピーを使用して **391 フレーム（0..=390）** を出力した。検証後、Application Support 側のプロジェクトファイルは元の 334 終了版に復元済み。
- ハーネス実行時の `0..=390` は環境変数で終了フレームを上書きしていた。最終報告の 391 フレーム UI 検証はハーネスなしで実施している。

---

## 6. `git diff --stat` と成果物

### 6.1 `git diff --stat`（HEAD 比・未コミット）

```text
 Cargo.lock                                      |  1 +
 crates/host-adapter/src/binary_path.rs          | 13 +++-
 crates/host-adapter/src/process.rs              |  5 ++
 crates/host-adapter/src/registry.rs             |  4 +
 crates/media/ffmpeg/Cargo.toml                  |  3 +
 crates/media/ffmpeg/src/colorconv.rs            | 12 ++-
 crates/media/ffmpeg/src/decoder/hw_device.rs    | 97 +++++++++++++++++++++++++
 crates/media/ffmpeg/src/decoder/mod.rs          | 11 +++
 crates/media/ffmpeg/src/decoder/open.rs         |  1 +
 crates/media/ffmpeg/src/decoder/staging_pool.rs | 16 +++-
 crates/media/ffmpeg/src/decoder/worker.rs       | 27 +++++++
 crates/media/ffmpeg/src/encoder.rs              |  7 +-
 crates/media/runtime/src/cache.rs               | 10 +++
 crates/media/support/src/lib.rs                 | 85 +++++++++++++---------
 i18n                                            |  0
 src/app/shortcuts.rs                            |  7 +-
 src/app/update.rs                               | 74 ++++++++++++-------
 src/audio/plugin_registry.rs                    |  8 +-
 src/audio/plugin_settings.rs                    | 10 +--
 src/infra/gpu_shared.rs                         | 28 +++++--
 src/project/mod.rs                              | 36 +++++++--
 src/renderer/effect_filter.rs                   |  2 +-
 src/renderer/pipeline/text.rs                   | 31 ++++++++
 src/ui/export_dialog.rs                         | 48 +++++++-----
 src/ui/properties/mod.rs                        | 21 +++++-
 src/ui/properties/sections.rs                   | 78 +++++++++++++++++++-
 src/ui/system_settings/helpers.rs               | 10 +--
 src/ui/timeline/actions.rs                      |  5 +-
 xtask/src/main.rs                               | 25 ++++++-
 29 files changed, 537 insertions(+), 138 deletions(-)
```

> 上記には Phase 2〜4 の既存未コミット変更を含む。Phase 4.5 で触れたのは `crates/media/runtime/src/cache.rs`、`crates/media/ffmpeg/src/decoder/mod.rs`、`crates/media/ffmpeg/src/decoder/staging_pool.rs`、`crates/media/ffmpeg/src/colorconv.rs` の 4 ファイル（HEAD 比で合計 +43 / -6）。このうち既存の `submit` ロックは Phase 3 からの分で、Phase 4.5 の新規追加は画像 `texture()` のロック、`map_async` ガード、`write_buffer` ガード、およびロック取得ヘルパー。

### 6.2 最終成果物

| 成果物 | サイズ | SHA-256 |
|---|---:|---|
| `target/bundle/NeoUtl.app` | 89.98 MB | — |
| `target/bundle/NeoUtl.app/Contents/MacOS/NeoUtl` | 32,598,480 bytes | `91bbd27825ceaa49859a906b03eada5a69b48468b6deedbda74bf2104b639dc5` |
| `target/bundle/NeoUtl-0.8.11-macos.dmg` | 41,882,128 bytes（39.94 MB） | `c814570bd5c6211960a697a56d37378d963c686706b3166872167cfad9cc78d0` |
| `target/bundle/NeoUtl-0.8.11-macos.zip` | 36,732,879 bytes（35.03 MB） | `41ce3ff3c1232d5573cfe60486fa8b5379f7e6f3c38ced8ec7253948193b3482` |
| `target/release/NeoUtl`（ベア） | 32,789,000 bytes | `b36eedcfa0dc591e04a8a7a88ab60abf5ce39874520dbbf4db8f12d798dacc24` |

- `codesign --verify --deep --strict target/bundle/NeoUtl.app` 合格（ad-hoc 署名）。
- パッケージは `cargo xtask package --release` で生成。
- ランタイムデータは `~/Library/Application Support/NeoUtl`、自己更新は無効のまま。

---

## 7. 今回の修正対象外（既存・無関係）

1. **起動時フォントプリロードの一時メモリ増加**
   - 371 フォント / 564 MB を読み込み、起動直後に一時的な high-water mark が数 GB に達する。
   - Phase 4.5 のロック修正とは無関係。Phase 5 で別途扱う。
2. **VRAM/RAM 予算取得失敗時のフォールバック**
   - `初期VRAM予算取得失敗 フォールバック容量=6`、`初期RAM予算取得失敗 フォールバック容量=64`。
   - 720p 全編再生の 301.47 MB はこの 64 フレーム RAM キャッシュが支配的。既存挙動で、停止修正の副作用ではない。
3. **書き出しダイアログの終了フレーム上限**
   - `total_frames` に依存する既存仕様。335 フレーム出力は停止ではなく上限によるもの。
4. **UI 書き出しと一時ハーネス書き出しの速度差**
   - ハーネスは 19〜22 秒、通常 UI はそれより長い。今回の修正範囲外の既存差として記録する。

---

## 8. Phase 5 への引き継ぎ

1. **プラグイン ABI の wgpu 重複解消（本命）**
   - 画像デコーダ dylib が wgpu / parking_lot を静的リンクする構造を改め、ホストの GPU ランタイムを共有する ABI（ハンドル経由の GPU 操作 API など）を設計する。
   - 今回のロック直列化は macOS の応急・最小修正であり、プラグインが増えるほどロック競合と保守コストが増す。
2. **回帰テスト化**
   - 動画＋画像プロジェクトの 391 フレーム書き出しを自動テストに追加する。
   - HW / SW 両方のデコード経路で実行できるようにする。
3. **書き出し範囲の仕様整理**
   - `total_frames` と画像オブジェクト終了フレームの関係を明確化し、UI の 335 / 391 フレーム差を仕様として文書化する。
4. **フォントプリロードのメモリ最適化**
   - 起動時 371 フォントの一括読み込みを見直し、遅延ロードまたは共有化を検討する。
5. **VRAM/RAM 予算プロバイダの macOS 対応**
   - 予算取得失敗時の 64 フレームフォールバックを改善し、全編再生の RSS を安定化する。
6. **配布**
   - 証明書・notarization・Apple ID は今回も未使用。配布時は別途対応が必要。

---

## 9. 制約遵守の確認

- HEAD は `0ead39350db55996810de53da79d02fbc98355f5` のまま。コミット・push・アップロードなし。
- Apple ID / 開発者証明書 / notarization / `sudo` は未使用。
- 一時ハーネスと一時診断 example は削除済み。`src/ui/mod.rs` / `src/app/mod.rs` に差分なし。
- LICENSE / copyright / アプリ名は変更していない。
- 無関係な既存不具合（フォントプリロード、RAM フォールバック等）は修正せず、記録のみ。
- Stage Manager は `GloballyEnabled=1` に復元済み。
- 検証用に起動した `NeoUtl` プロセスはすべて終了済み。
