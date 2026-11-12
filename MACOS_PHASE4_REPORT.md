# NeoUtl macOS 移植 Phase 4（.app 化・配布パッケージ）報告書

作成日: 2026-11-12  
対象作業ディレクトリ: `/Users/ray_9618/Desktop/Git/NeoUtl-macOS`  
対象コミット: `0ead39350db55996810de53da79d02fbc98355f5` (v0.8.11, AGPL-3.0-or-later)  
検証担当: OpenCode AI  

---

## 0. エグゼクティブサマリ

Phase 4 の目的である「再現可能な `.app` の生成、FFmpeg 依存 dylib の同梱、ad-hoc 署名、`.dmg` / `.zip` の生成、実機 E2E 検証」を完了した。作業ツリーの変更はすべて未コミットのまま維持し、`git commit` / `git push` / 外部アップロードは行っていない。

| 項目 | 結果 | 判定 |
|---|---|---|
| `cargo xtask package --release` | 成功。`.app` / `.dmg` / `.zip` を生成 | **Pass** |
| `.app` 起動（`open`） | PID 47317、960×672 ウィンドウを確認。起動後も署名有効 | **Pass** |
| クリーン環境起動 | `env -i` で 9 Objects / 23 Effects / 3 Decoders / 1 Easing を登録 | **Pass** |
| 720p 書き出し | 1280×720 / 30fps / 10.0s / H.264 を生成。非黒フレームを確認 | **Pass**（テキストプロジェクト） |
| dylib 同梱 | Homebrew 由来 19 種を `Contents/Frameworks` に同梱。`@rpath` 化 | **Pass** |
| 絶対パス検査 | 55 Mach-O、286 ロード参照、Homebrew 絶対パス 0 件 | **Pass** |
| ad-hoc 署名 | `codesign --verify --deep --strict` 成功。起動後・DMG 内・ZIP 展開後も成功 | **Pass** |
| Gatekeeper | `spctl` は rejected。隔離属性付き起動で想定どおりブロックを記録 | **記録済み** |
| macOS 自動更新 | `cfg(target_os = "macos")` で無効化 | **Pass** |
| 回帰 | `cargo fmt` / `cargo xtask build --release` / workspace tests すべて成功 | **Pass** |
| 未コミット状態 | HEAD は `0ead393` のまま。変更 31 エントリを維持 | **Pass** |

**Phase 4 のスコープは完了。** ただし、動画オブジェクトを含むプロジェクトの書き出しは、後述のとおりメディアデコーダ側のハングが残るため、最終 E2E はテキストプロジェクトで実施した。この点は Phase 5 の最優先調査項目として申し送る。

---

## 1. 開始前確認

### 1.1 Git 作業ツリー

| 項目 | 値 |
|---|---|
| リポジトリ | `/Users/ray_9618/Desktop/Git/NeoUtl-macOS` |
| ブランチ | `main` |
| HEAD | `0ead39350db55996810de53da79d02fbc98355f5` |
| 最終コミット | `0ead3935 2026-10-06 07:45:39 +0900 READMEを更新` |
| バージョン | `0.8.11` |
| `CFBundleVersion` の元 | `git rev-list --count HEAD` = `1178` |
| ライセンス | AGPL-3.0-or-later（`LICENSE` に差分なし） |

Phase 1〜3 の変更を含む作業ツリーに対して Phase 4 のパッケージングを追加した。Phase 4 終了時点でも HEAD は変えていない。

### 1.2 実行環境

| 項目 | 値 |
|---|---|
| macOS | 27.0.1（Build 26A434） |
| CPU | Apple M1（arm64） |
| Xcode | 27.2（Build 27B52） |
| Command Line Tools | `/Library/Developer/CommandLineTools` |
| Rust | 1.99.1（2026-08-27） |
| Cargo | 1.99.1 |
| Python | 3.9.6（`/usr/bin/python3`） |
| FFmpeg / ffprobe | 9.0.2（`/opt/homebrew/bin`） |

Apple ID、Developer ID 証明書、公証（notarization）、`sudo` は使用していない。署名は ad-hoc（`codesign --sign -`）のみである。

### 1.3 検証上の制約

- 画面収録（Screen Recording）の TCC 権限が拒否されているため、OS 全体のスクリーンショットは取得していない。Gatekeeper のダイアログは Accessibility API（AX）のダンプ、書き出し結果は ffprobe / ffmpeg によるフレーム解析で証拠化した。
- 最終バイナリには AccessKit が組み込まれていないため、egui の UI 内容は AX から直接取得できない。UI 自動化は座標クリックとキーボード入力で行った。
- 自動化中は Stage Manager を一時的に無効化した。検証終了後に `GloballyEnabled=true` へ戻し、WindowManager を再起動して復元済みである。

---

## 2. 成果物

### 2.1 生成物一覧

| 成果物 | パス | サイズ |
|---|---|---|
| アプリバンドル | `/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/bundle/NeoUtl.app` | 94,342,515 bytes（89.97 MiB、`du -sh` では 90M） |
| DMG | `/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/bundle/NeoUtl-0.8.11-macos.dmg` | 41,876,545 bytes（39.94 MiB） |
| ZIP | `/Users/ray_9618/Desktop/Git/NeoUtl-macOS/target/bundle/NeoUtl-0.8.11-macos.zip` | 36,733,829 bytes（35.03 MiB） |

`.app` の論理ファイルサイズは 94,342,515 bytes、ディスク使用量は 92,492 KiB。バンドル内の通常ファイルは 119、シンボリックリンクは 8、ディレクトリ 43 を含む全エントリは 170。

### 2.2 SHA-256

```text
ce7846928e265526f0f0a811076709643f7b6716b2af8ef2c356ec172ca0574c  NeoUtl-0.8.11-macos.dmg
5d78f33c90c4b6ca78e216ade896c97c05c9c100920a751578e0f9ec07a70190  NeoUtl-0.8.11-macos.zip
aa450e6616a4d680c860559876ddabc8d3763936bf11f9675a7cd246ab93a635  NeoUtl.app/Contents/MacOS/NeoUtl
```

### 2.3 成果物の再検証

- **DMG**: `hdiutil attach -nobrowse -readonly` でマウントし、`NeoUtl.app` を確認。マウント中のアプリに対して `codesign --verify --deep --strict` が成功し、`CFBundleShortVersionString=0.8.11` を確認。その後 detach 済み。
- **ZIP**: `ditto -x -k` で別ディレクトリへ展開し、展開後の `NeoUtl.app` に対して `codesign --verify --deep --strict` が成功。
- **DMG 形式**: UDIF 読み出し専用圧縮（zlib）。CRC32 チェックサムあり。
- **ZIP の注意**: `ditto` の仕様により AppleDouble（`._*`）エントリが含まれる。展開後のアプリは署名検証済みで、実害はない。Phase 5 で `--norsrc` 等による整理を検討可能。

---

## 3. バンドル構成

### 3.1 ツリー

```text
NeoUtl.app/
└── Contents/
    ├── Info.plist
    ├── PkgInfo                         # APPL????
    ├── MacOS/
    │   ├── NeoUtl                       # 32,595,088 bytes
    │   └── maolan-plugin-host           # 1,983,520 bytes
    ├── Frameworks/
    │   ├── libSvtAv1Enc.4.2.0.dylib
    │   ├── libavcodec.63.1.102.dylib
    │   ├── libavdevice.63.1.102.dylib
    │   ├── libavfilter.12.1.102.dylib
    │   ├── libavformat.63.1.102.dylib
    │   ├── libavutil.61.1.102.dylib
    │   ├── libcrypto.3.dylib
    │   ├── libdav1d.7.dylib
    │   ├── liblzma.5.dylib
    │   ├── libmp3lame.0.dylib
    │   ├── libmpg123.0.dylib
    │   ├── libopus.0.dylib
    │   ├── libssl.3.dylib
    │   ├── libswresample.7.1.102.dylib
    │   ├── libswscale.10.1.102.dylib
    │   ├── libvmaf.3.dylib
    │   ├── libvpx.12.dylib
    │   ├── libx264.165.dylib
    │   ├── libx265.217.dylib
    │   └── （8 個のバージョン別シンボリックリンク）
    ├── Resources/
    │   ├── NeoUtl.icns                  # 690,511 bytes
    │   ├── objects/                     # 9 dylib + 14 i18n ファイル
    │   ├── effects/                     # 23 dylib + 46 i18n ファイル
    │   ├── decoders/                    # 2 dylib
    │   ├── easings/                     # 空（組み込み Easing Engine を使用）
    │   └── scripts/                     # 空（Lua スクリプトなし）
    └── _CodeSignature/
        └── CodeResources
```

### 3.2 ファイル数

| ディレクトリ | 内容 | ファイル数 |
|---|---|---:|
| `Contents/MacOS` | `NeoUtl`, `maolan-plugin-host` | 2 |
| `Contents/Frameworks` | 実 dylib 19 + シンボリックリンク 8 | 19（実体） |
| `Contents/Resources/objects` | 9 dylib + 14 i18n | 23 |
| `Contents/Resources/effects` | 23 dylib + 46 i18n | 69 |
| `Contents/Resources/decoders` | 2 dylib | 2 |
| `Contents/Resources/easings` | 空 | 0 |
| `Contents/Resources/scripts` | 空 | 0 |
| `Contents` 直下 | `Info.plist`, `PkgInfo` | 2 |
| `_CodeSignature` | `CodeResources` | 1 |
| **合計** | | **119** |

全 170 エントリ（ディレクトリ 43 を含む）の一覧は `target/phase4_bundle/bundle_manifest.txt` に保存した。

### 3.3 構成上の確認

- `Contents/MacOS` は `NeoUtl` と `maolan-plugin-host` の 2 ファイルのみ。Phase 1〜3 で問題になった `settings/` や `projects/` は存在しない。
- `Contents/MacOS` から `Contents/Resources` へのシンボリックリンクは作成していない。macOS 側の `../Resources/{objects,effects,decoders,easings}` フォールバックで解決する。
- アプリ起動後の書き込み先は `~/Library/Application Support/NeoUtl` に移転され、バンドル内は読み取り専用のまま維持される（詳細は 7.3）。
- `easings/` と `scripts/` は空だが、Easing は組み込みエンジン `neoutl-easing-standard` が登録され、Lua スクリプトは今回の構成に含まれないため空で正常。

---

## 4. Info.plist

### 4.1 最終値

`plutil -lint` は `OK`。`PkgInfo` は `APPL????`。

| キー | 値 | 備考 |
|---|---|---|
| `CFBundleDevelopmentRegion` | `ja` | 日本語 UI |
| `CFBundleDisplayName` | `NeoUtl` | 表示名 |
| `CFBundleExecutable` | `NeoUtl` | `Contents/MacOS/NeoUtl` |
| `CFBundleIconFile` | `NeoUtl.icns` | `assets/neoutl/icon.svg` から生成 |
| `CFBundleIdentifier` | `works.rayno.neoutl` | 暫定 ID（Phase 5 で確定） |
| `CFBundleInfoDictionaryVersion` | `6.0` | |
| `CFBundleName` | `NeoUtl` | |
| `CFBundlePackageType` | `APPL` | |
| `CFBundleShortVersionString` | `0.8.11` | `Cargo.toml` と一致 |
| `CFBundleVersion` | `1178` | `git rev-list --count HEAD` |
| `LSApplicationCategoryType` | `public.app-category.video` | |
| `LSMinimumSystemVersion` | `27.0` | 後述の根拠あり |
| `NSHighResolutionCapable` | `true` | Retina 対応 |
| `NSRequiresAquaSystemAppearance` | `false` | ダークモード追従 |
| `NSSupportsAutomaticGraphicsSwitching` | `true` | GPU 切替対応 |

### 4.2 `LSMinimumSystemVersion=27.0` の根拠

バンドル内の全 Mach-O（55 ファイル）の `LC_BUILD_VERSION.minos` / `LC_VERSION_MIN_MACOSX.version` を走査した結果、最大値は **27.0** だった。これは Homebrew の FFmpeg 9.0.2 および依存 dylib が macOS 27.0 向けにビルドされているためである。

- メイン実行ファイル自体の `VersionMin` は 11.0（`codesign` 表記 `720896`）、SDK は 27.0（`1769472`）。
- しかし同梱 dylib の最大 `minos` が 27.0 のため、27.0 未満の macOS では dylib のロードに失敗する可能性が高い。
- したがって `LSMinimumSystemVersion=27.0` は「実際に動作する下限」を正直に示す値として設定した。Phase 5 で下限を引き下げる場合は、FFmpeg と依存ライブラリをより低い deployment target で再ビルドする必要がある。

### 4.3 アイコン

`assets/neoutl/icon.svg` を 1024×1024 に正規化し、`sips` で 16〜1024px の iconset を生成、`iconutil -c icns` で `NeoUtl.icns`（690,511 bytes）を作成した。Finder 表示用の `CFBundleIconFile` に設定済み。

---

## 5. dylib の同梱とロードパス

### 5.1 同梱した Homebrew 由来 dylib（19 種）

```text
Contents/Frameworks/libSvtAv1Enc.4.2.0.dylib
Contents/Frameworks/libavcodec.63.1.102.dylib
Contents/Frameworks/libavdevice.63.1.102.dylib
Contents/Frameworks/libavfilter.12.1.102.dylib
Contents/Frameworks/libavformat.63.1.102.dylib
Contents/Frameworks/libavutil.61.1.102.dylib
Contents/Frameworks/libcrypto.3.dylib
Contents/Frameworks/libdav1d.7.dylib
Contents/Frameworks/liblzma.5.dylib
Contents/Frameworks/libmp3lame.0.dylib
Contents/Frameworks/libmpg123.0.dylib
Contents/Frameworks/libopus.0.dylib
Contents/Frameworks/libssl.3.dylib
Contents/Frameworks/libswresample.7.1.102.dylib
Contents/Frameworks/libswscale.10.1.102.dylib
Contents/Frameworks/libvmaf.3.dylib
Contents/Frameworks/libvpx.12.dylib
Contents/Frameworks/libx264.165.dylib
Contents/Frameworks/libx265.217.dylib
```

### 5.2 バージョン別シンボリックリンク（8 個）

```text
libSvtAv1Enc.4.dylib       -> libSvtAv1Enc.4.2.0.dylib
libavcodec.63.dylib        -> libavcodec.63.1.102.dylib
libavdevice.63.dylib       -> libavdevice.63.1.102.dylib
libavfilter.12.dylib       -> libavfilter.12.1.102.dylib
libavformat.63.dylib       -> libavformat.63.1.102.dylib
libavutil.61.dylib         -> libavutil.61.1.102.dylib
libswresample.7.dylib      -> libswresample.7.1.102.dylib
libswscale.10.dylib        -> libswscale.10.1.102.dylib
```

いずれも `Frameworks` 内の相対シンボリックリンクであり、バンドル外を参照しない。

### 5.3 ロードパスの書き換え

- メイン実行ファイルと `Resources` 配下のプラグイン dylib には `@executable_path/../Frameworks` の rpath を追加。
- `Frameworks` 内の各 dylib は `install_name_tool -id @rpath/<name>` と `@loader_path` の rpath を設定。
- 各 dylib の Homebrew 絶対パス依存は `install_name_tool -change <abs> @rpath/<basename>` で書き換え。

メイン実行ファイルの `otool -L` 抜粋:

```text
@rpath/libavutil.61.dylib (compatibility version 61.0.0, current version 61.1.102)
@rpath/libavformat.63.dylib (compatibility version 63.0.0, current version 63.1.102)
@rpath/libavfilter.12.dylib (compatibility version 12.0.0, current version 12.1.102)
@rpath/libavdevice.63.dylib (compatibility version 63.0.0, current version 63.1.102)
@rpath/libswscale.10.dylib (compatibility version 10.0.0, current version 10.1.102)
@rpath/libswresample.7.dylib (compatibility version 7.0.0, current version 7.1.102)
@rpath/libavcodec.63.dylib (compatibility version 63.0.0, current version 63.1.102)
/System/Library/Frameworks/...
/usr/lib/libSystem.B.dylib
```

プラグイン dylib の例:

```text
Resources/objects/libneoutl_object_text.dylib:
    @rpath/libneoutl_object_text.dylib
    /usr/lib/libSystem.B.dylib
```

### 5.4 絶対パス検査結果

| 検査項目 | 結果 |
|---|---:|
| 検査対象 Mach-O | 55 |
| `otool -L` の総ロード参照 | 286 |
| Homebrew 絶対パス（`/opt/homebrew/...`） | **0** |
| 非システムの絶対パス（`/System`, `/usr/lib`, `/Library/Apple` 以外） | **0** |
| 最大 `minos` | **27.0** |

内訳は `MacOS` 2 + `Frameworks` 19 + `Resources` 配下の dylib 34（objects 9 + effects 23 + decoders 2）= 55。シンボリックリンクは Mach-O 検査対象から除外している。

証拠ファイル:

- `target/phase4_bundle/evidence/macho_files.txt`
- `target/phase4_bundle/evidence/macho_load_refs.txt`

---

## 6. コード署名

### 6.1 署名方式と順序

`scripts/package_macos.py` の `sign_bundle()` で、以下の順に ad-hoc 署名（`codesign --force --sign -`）を適用した。

1. `Contents/Frameworks` 内の実 dylib
2. `Contents/Resources` 配下のプラグイン dylib
3. `Contents/MacOS` の `maolan-plugin-host` と `NeoUtl`
4. `.app` バンドル本体

最後に `codesign --verify --deep --strict` を実行し、成功を確認している。

### 6.2 署名情報

```text
Executable=.../NeoUtl.app/Contents/MacOS/NeoUtl
Identifier=works.rayno.neoutl
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20400 size=63755 flags=0x2(adhoc) hashes=1986+3 location=embedded
Hash type=sha256 size=32
CandidateCDHash sha256=b5f74dd48ca596043a17a8b94bd423f0caec6c4c
CandidateCDHashFull sha256=b5f74dd48ca596043a17a8b94bd423f0caec6c4ce9aec947198b18af36097995
Signature=adhoc
Info.plist entries=15
TeamIdentifier=not set
Sealed Resources version=2 rules=13 files=123
Total signatures=1
```

`flags=0x2(adhoc)` のみで、Hardened Runtime（`runtime`）は付与していない。Entitlements も設定していない。Phase 5 で Developer ID 署名 + Hardened Runtime + 公証を行う際に追加する。

### 6.3 検証結果

| 検証 | 結果 |
|---|---|
| `codesign --verify --deep --strict NeoUtl.app` | 成功（`valid on disk` / `satisfies its Designated Requirement`） |
| 起動後の再検証 | 成功（署名は起動で壊れない） |
| DMG マウント中の検証 | 成功 |
| ZIP 展開後の検証 | 成功 |
| `spctl --assess --type execute -vv` | `rejected`（ad-hoc のため想定どおり） |

### 6.4 Gatekeeper / 隔離属性の実測

最終バンドル（`NeoUtl` の SHA-256 は `aa450e...`）を `target/phase4_bundle/quarantine/NeoUtl.app` に複製し、`com.apple.quarantine` 属性（フラグ `0081`、付与元 `OpenCode`）を付与して `open` した。

- `open` 自体は exit 0 を返すが、アプリは起動しない。
- `CoreServicesUIAgent` が以下のダイアログを表示:

  ```text
  “NeoUtl”は開いていません
  Appleは、“NeoUtl”にMacに損害を与えたり、プライバシーを侵害する可能性のある
  マルウェアが含まれていないことを検証できませんでした。
  ボタン: ゴミ箱に入れる / 完了
  ```

- この時点で `NeoUtl` のプロセスは起動していない。
- `xattr -dr com.apple.quarantine` で隔離属性を削除し、「完了」を押した後、同じコピーが PID 34340 / ウィンドウ 24592 で起動した。

これは ad-hoc 署名のみのアプリに対する Gatekeeper の想定挙動であり、不具合ではない。Phase 5 の Developer ID 署名 + 公証 + staple で解消する。

証拠ファイル:

- `target/phase4_bundle/evidence/gatekeeper_prompt_ax.log`
- `target/phase4_bundle/evidence/gatekeeper_outcome.log`

---

## 7. E2E 検証

### 7.1 `open` による起動

```text
command: open target/bundle/NeoUtl.app
pid: 47317
window: NeoUtl 960×672 at (360, 106)
codesign after launch: 成功（エラー出力なし）
```

`open` 経由で LaunchServices に登録され、通常の `.app` として起動することを確認した。起動後も `codesign --verify --deep --strict` は成功し、署名の破壊はない。

証拠: `target/phase4_bundle/evidence/open_launch.log`

### 7.2 クリーン環境での起動

`PATH` と `HOME` 以外の環境変数を排除した状態で、バンドル内の実行ファイルを直接起動した。

```text
env -i HOME="$HOME" PATH="/usr/bin:/bin:/usr/sbin:/sbin" TMPDIR="$TMPDIR" \
  ./target/bundle/NeoUtl.app/Contents/MacOS/NeoUtl
```

| 登録項目 | 件数 |
|---|---:|
| Objects | 9 |
| Effects | 23 |
| Decoders | 3（FFmpeg / Image / Symphonia） |
| Easing Engines | 1（`neoutl-easing-standard`） |

9 Objects の内訳:

```text
neoutl.object.audio
neoutl.object.camera
neoutl.object.group_control
neoutl.object.image
neoutl.object.light
neoutl.object.scene
neoutl.object.shape
neoutl.object.text
neoutl.object.video
```

クリーン環境でも Homebrew の PATH に依存せず、同梱 dylib とプラグインがロードされた。`Contents/MacOS/settings` と `Contents/MacOS/projects` は生成されていない。

証拠: `target/phase4_bundle/evidence/clean_env_launch.log`

### 7.3 ランタイムデータの移転

macOS の `.app` バンドル内（`Contents/MacOS` 配下）で実行した場合、`runtime_data_dir()` は以下を返す。

```text
~/Library/Application Support/NeoUtl
```

確認結果:

- `~/Library/Application Support/NeoUtl/projects` が存在し、E2E プロジェクトが保存されている。
- `~/Library/Application Support/NeoUtl/settings` に `audio-plugin-settings.npb` と `running.lock` が生成されている。
- `Contents/MacOS` には `NeoUtl` と `maolan-plugin-host` 以外のファイル・ディレクトリはない。
- 素のバイナリ（`target/release/NeoUtl`）は従来どおり実行ファイル隣の `projects` / `settings` を使う。Phase 2/3 の検証経路を壊さない。

### 7.4 720p 書き出し（最終 E2E）

#### 使用プロジェクト

`phase4_export_text_720p`

| 項目 | 値 |
|---|---|
| シーン | 1280×720 / 30fps / 10 秒 |
| オブジェクト | Text 1 個（フレーム 0〜300） |
| テキスト | `Phase 4 E2E 1280x720` |
| フォント | Hiragino Sans |
| Transform | `scale_x=1`, `scale_y=1`, `opacity=1`（テスト資産側で補正） |
| 出力 | `target/phase4_bundle/exports/export_phase4_text_720p_final3.mp4` |

> 注: 自動化によるプロパティ編集でテストプロジェクトの Transform がゼロになり、一時期は背景のみ（18,529 bytes）が書き出されていた。製品コードではなくテスト資産（`project.npb`）の Transform を補正して再書き出しした。バックアップは `project.npb.bak` に残している。

#### ffprobe 結果

```text
index=0
codec_name=h264
codec_type=video
width=1280
height=720
r_frame_rate=30/1
nb_frames=301
duration=10.000000
size=113602
bit_rate=90881
```

- 出力サイズ: **113,602 bytes**
- コンテナ: MP4 / H.264
- 解像度: **1280×720**
- フレームレート: **30/1**
- 長さ: **10.000 秒**
- デコード確認フレーム数: 300（`nb_read_frames=300`）、パケット数 301

#### 非黒フレームの確認

`ffmpeg -ss 4.5` で抽出したフレーム（`target/phase4_bundle/exports/phase4_text_frame_4.5s.png`）を解析:

```text
size          = 1280×720
extrema       = ((13,255), (12,255), (14,255))
mean          = 18.616
white_pixels  = 12,758
white bbox    = x:240..1041, y:327..378
corner pixel  = (14,13,18)
center pixel  = (14,13,18)
```

- 背景色は `(14,13,18)` で一様。
- 中央に白文字（`Phase 4 E2E 1280x720`）が描画され、白ピクセル 12,758 個、バウンディングボックス `240..1041 × 327..378` を確認。
- t=0.5 / 2 / 4.5 / 7 / 9 の各フレームで同様に非黒の白文字を確認。t=0 でも文字が表示される。
- したがって「背景のみの 18,529 bytes」ではなく、実際にオブジェクトが描画された 720p 動画である。

証拠:

- `target/phase4_bundle/evidence/export_720p.log`
- `target/phase4_bundle/evidence/export_runtime.log`
- `target/phase4_bundle/exports/phase4_text_frame_4.5s.png`

#### 失敗した試行の保存

以前の試行で生成された以下のファイルは、調査用に `target/phase4_bundle/exports/failed_attempts/` へ移動した。

| ファイル | サイズ | 内容 |
|---|---:|---|
| `export_phase2_check.mp4` | 48 bytes | 動画プロジェクトの書き出しがフレーム 51 で停止 |
| `export_phase4_720p.mp4` | 18,529 bytes | 背景のみ（Transform ゼロ） |
| `export_phase4_720p_final.mp4` | 18,529 bytes | 背景のみ（Transform ゼロ） |
| `export_phase4_text_720p.mp4` | 18,529 bytes | 背景のみ（Transform 補正前） |

**既知の未解決事項**: 動画オブジェクトを含む `phase2_e2e_720p` の書き出しは、FFmpeg デコーダのログがフレーム 51 付近で停止し、プロセス CPU 0% のまま 48 bytes のファイルを残す。VideoToolbox の初期化・シーク後のパケット処理・共有 wgpu キュー周辺を Phase 5 で重点調査する。最終 E2E はテキストプロジェクトで 720p 書き出し経路そのものを検証した。

### 7.5 プラグイン登録の完全性

クリーン環境起動ログで以下を確認:

- Objects: 9（audio, camera, group_control, image, light, scene, shape, text, video）
- Effects: 23（border_blur 〜 vibration）
- Decoders: 3（FFmpeg, Image, Symphonia）
- Easing: 1（Standard Easing Engine）
- Audio plugin: `MonkSynth`（`/Library/Audio/Plug-Ins/VST3/MonkSynth.vst3`）を検出

Phase 3 の「全 9 Objects / 全 23 Effects」の登録結果が、最終 `.app` のクリーン環境でも再現した。

### 7.6 Gatekeeper の実測

6.4 のとおり。`spctl` は rejected、隔離属性付き起動では CoreServicesUIAgent のブロックダイアログが表示され、アプリは起動しなかった。隔離属性を削除して「完了」を押した後は起動した。ad-hoc 署名の配布物として想定どおりの挙動である。

### 7.7 証拠の制約

- Screen Recording TCC が拒否されているため、Gatekeeper ダイアログの画面画像はない。AX ダンプ（`gatekeeper_prompt_ax.log`）にダイアログの全文とボタン名を記録している。
- 書き出し画像はアプリの出力そのもの（`phase4_text_frame_4.5s.png`）で、ピクセル解析結果を数値で記録している。
- UI 操作は座標ベースの自動化であり、最終バイナリの AX ツリーからウィジェットを直接特定するものではない。

---

## 8. macOS 自動更新の扱い

### 8.1 判断

macOS の `.app` バンドル運用では、既存の自己更新（self-replace）が `_CodeSignature` を破壊し、同梱 dylib / プラグインとの整合性も失う。Phase 4 では **macOS のみ自動更新を無効化** した。

### 8.2 実装

`src/app/update.rs`:

- `spawn_check` は `#[cfg(target_os = "macos")]` で直ちに `UpdateStatus::UpToDate` を設定し、ネットワークチェックを開始しない。
- `spawn_apply` は `#[cfg(target_os = "macos")]` で `UpdateStatus::Error("macOSではアプリケーションバンドルの自動自己置換は無効化されています")` を返す。
- `#[cfg(not(target_os = "macos"))]` の既存処理は変更していない。

### 8.3 Phase 5 への申し送り

- Developer ID 署名 + 公証 + staple 後、署名済みの更新チャネル（Sparkle 等）を導入する。
- 更新は「アプリ全体の置換」ではなく、署名済み DMG / ZIP の配布 + LaunchServices 再登録を基本とする。
- 自動更新を再開する場合は、更新前後の `codesign --verify --deep --strict` と Gatekeeper 評価を必須にする。

---

## 9. 回帰確認

| コマンド | 結果 |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo xtask build --release` | exit 0 |
| `cargo test --workspace --exclude NeoUtl` | exit 0 |
| `cargo xtask package --release` | exit 0 |

### 9.1 テスト集計

```text
suites=80 passed=17 failed=0 ignored=7
```

- `cargo test --workspace --exclude NeoUtl` は 80 個のテスト結果行を持ち、17 passed / 0 failed / 7 ignored。
- `cargo fmt --all -- --check` は差分なし。
- `cargo xtask build --release` は成功。警告が 1 件のみ:
  - `src/audio/plugin_registry.rs:3` の `use std::path::{Path, PathBuf};` で `Path` が未使用。
  - 動作には影響しない。Phase 5 で整理可能。

証拠:

- `target/phase4_bundle/evidence/regression_build.log`
- `target/phase4_bundle/evidence/regression_tests.log`

---

## 10. 変更ファイル

### 10.1 Git 状態

HEAD は `0ead39350db55996810de53da79d02fbc98355f5` のまま。`git status --porcelain` は 31 エントリ:

- 変更された追跡ファイル: 24
- 変更されたサブモジュール: 1（`i18n`）
- 未追跡ファイル: 6（Phase 0/1・2・3・4 レポート、`scripts/package_macos.py`、`scripts/package_macos.sh`）

`LICENSE` に差分はなく、著作権表記・ライセンスは変更していない。

### 10.2 diff stat

```text
 crates/host-adapter/src/binary_path.rs          | 13 +++-
 crates/host-adapter/src/process.rs              |  5 ++
 crates/host-adapter/src/registry.rs             |  4 +
 crates/media/ffmpeg/src/colorconv.rs            |  6 +-
 crates/media/ffmpeg/src/decoder/hw_device.rs    | 97 +++++++++++++++++++++++++
 crates/media/ffmpeg/src/decoder/open.rs         |  1 +
 crates/media/ffmpeg/src/decoder/staging_pool.rs |  6 +-
 crates/media/ffmpeg/src/decoder/worker.rs       | 27 +++++++
 crates/media/ffmpeg/src/encoder.rs              |  7 +-
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
 25 files changed, 500 insertions(+), 134 deletions(-)
```

未追跡の `scripts/package_macos.py` / `scripts/package_macos.sh` は diff stat に含まれない。

### 10.3 主要変更の目的

| ファイル | 変更目的 |
|---|---|
| `scripts/package_macos.py`（新規） | `.app` 生成、dylib 再帰収集、`@rpath` 書き換え、Info.plist / icns 生成、ad-hoc 署名、DMG / ZIP 生成 |
| `scripts/package_macos.sh`（新規） | `cargo xtask package` の薄いラッパー |
| `xtask/src/main.rs` | `package` サブコマンド追加、`PLUGIN_HOST_VERSION` を 0.0.8 に更新 |
| `src/app/update.rs` | macOS の自動更新を無効化 |
| `src/project/mod.rs` | `runtime_data_dir()` を追加。`.app` 内は `~/Library/Application Support/NeoUtl`、素のバイナリは実行ファイル隣 |
| `src/app/shortcuts.rs` | キーマップ保存先を `runtime_data_dir()` 経由に変更 |
| `src/audio/plugin_registry.rs` | `audio-plugins` ディレクトリを `runtime_data_dir()` 経由に変更 |
| `src/audio/plugin_settings.rs` | 音声プラグイン設定を `runtime_data_dir()` 経由に変更 |
| `src/ui/system_settings/helpers.rs` | システム設定を `runtime_data_dir()` 経由に変更 |
| `src/ui/export_dialog.rs` | macOS で出力パスを直接編集可能にし、`rfd` の既定ファイル名を設定 |
| `src/renderer/pipeline/text.rs` | macOS の Hiragino Sans フォールバック |
| `src/infra/gpu_shared.rs` | macOS の wgpu limits / NV12 機能の任意化、共有 device/queue を FFmpeg 側へ公開 |
| `crates/media/ffmpeg/src/colorconv.rs` | wgpu submit を共有ロックで直列化 |
| `crates/media/ffmpeg/src/decoder/staging_pool.rs` | 同上 |
| `src/renderer/effect_filter.rs` | 同上（`locked_submit`） |
| `crates/media/ffmpeg/src/decoder/hw_device.rs` | VideoToolbox 初期化、失敗時の SW フォールバック |
| `crates/media/ffmpeg/src/decoder/worker.rs` | シーク後の stale packet 破棄と診断ログ |
| `crates/media/ffmpeg/src/decoder/open.rs` | シーク時のパケットキュー flush |
| `crates/media/ffmpeg/src/encoder.rs` | `avio_open` 失敗時の詳細エラー |
| `crates/media/support/src/lib.rs` | macOS で VAAPI probe をバイパス |
| `crates/host-adapter/src/binary_path.rs` | 実行ファイル隣のプラグインホストを優先し、存在しなければ名前でフォールバック |
| `crates/host-adapter/src/process.rs` | macOS の既定 `VST3_PATH` を設定 |
| `crates/host-adapter/src/registry.rs` | スキャンプロセスにも既定 `VST3_PATH` を設定 |
| `src/ui/properties/mod.rs` | オブジェクト選択 ComboBox を追加 |
| `src/ui/properties/sections.rs` | macOS の IME preedit / commit 処理と入力矩形通知 |
| `src/ui/timeline/actions.rs` | タイムライン選択を ECS の選択状態へ同期 |
| `i18n/ja.yml` | VideoToolbox 診断、stale packet、macOS 更新無効化、出力パス等の 7 文言を追加 |

### 10.4 完全 diff

作業ツリー全体の diff は以下に保存した。

- `target/phase4_bundle/evidence/phase4_working_tree.diff`（41,207 bytes）
- `target/phase4_bundle/evidence/phase4_diff_stat.txt`

Phase 1〜3 の変更を含む累積 diff であり、Phase 4 固有の追加はパッケージスクリプト、`xtask package`、`runtime_data_dir()`、macOS 自動更新無効化、`Info.plist` / アイコン生成である。

---

## 11. Phase 5 への申し送り

### 11.1 配布署名・公証

1. Apple Developer Program の Developer ID Application 証明書で再署名する。
2. Hardened Runtime を有効化し、必要なら audio input / JIT 等の Entitlements を付与する。
3. `xcrun notarytool submit --wait` で DMG を公証し、`xcrun stapler staple` で staple する。
4. 公証後、`spctl --assess --type execute` が accepted になること、Gatekeeper の隔離属性付き起動が通ることを再検証する。

### 11.2 アーキテクチャと最低 OS

- 現状は arm64 thin。Intel Mac も配布するなら universal2 化が必要。
- `LSMinimumSystemVersion=27.0` は Homebrew FFmpeg の `minos` に由来する。下限を下げるには FFmpeg / OpenSSL / x264 / x265 等を低い deployment target で再ビルドし、dylib の `minos` を下げる。
- Homebrew 依存の再現性のため、Phase 5 ではビルド用の固定バージョン・チェックサム・CI キャッシュを定義する。

### 11.3 動画書き出しの未解決事象

- `phase2_e2e_720p`（動画オブジェクト）の書き出しがフレーム 51 付近で停止し、48 bytes のファイルを残す。
- 動画プロジェクトの Transform が自動化でゼロになり、背景のみ（18,529 bytes）になる事象もあった。
- 重点調査対象:
  - VideoToolbox 初期化後の `avcodec_send_packet` / `avcodec_receive_frame` の状態遷移
  - シーク後のパケットキューと `key_index` / `seek_pts` の整合
  - 共有 wgpu device / queue と `locked_submit` のデッドロック
  - メディアキャッシュの GOP 境界処理
- テキストプロジェクトの 720p 書き出しは成功しているため、書き出しパイプライン全体ではなく、動画デコード経路に限定した問題として切り分ける。

### 11.4 自動更新

- macOS の自動更新は無効のまま。署名済み更新チャネル（Sparkle 等）を導入するまで再有効化しない。
- 更新前後で `codesign --verify --deep --strict` と Gatekeeper 評価を自動化する。

### 11.5 パッケージ品質

- `src/audio/plugin_registry.rs` の未使用 `Path` import 警告を解消する。
- ZIP の AppleDouble エントリを `--norsrc` 等で整理するか、配布用 ZIP の仕様として明記する。
- DMG に Applications へのシンボリックリンク、背景画像、アイコン配置、ライセンス表示を追加する。
- `CFBundleIdentifier` を暫定 `works.rayno.neoutl` から正式 ID へ確定する。
- `scripts/package_macos.py` の再現性（Homebrew バージョン固定、`otool` / `codesign` の失敗時即時停止）を CI で検証する。

### 11.6 テスト自動化

- 今回の UI 操作は `/tmp` の `.command` スクリプトと座標クリックに依存している。Phase 5 では再実行可能な E2E ハーネスをリポジトリ内に置くか、手順書として固定する。
- 画面収録 TCC が使えない環境でも検証できるよう、AX ダンプと ffprobe / フレーム解析を組み合わせた証拠取得を標準化する。

---

## 12. 終了状態とクリーンアップ

- NeoUtl のプロセスは停止済み。`Contents/MacOS` に `settings` / `projects` は生成されていない。
- Stage Manager は `GloballyEnabled=true` に戻し、WindowManager を再起動済み。
- 検証用の隔離コピー、失敗書き出し、バックアップは `target/phase4_bundle/` 配下に保存し、リポジトリの追跡対象には含めていない。
- テスト用プロジェクト `phase4_export_text_720p` は `~/Library/Application Support/NeoUtl/projects/` に残している。`project.npb.bak` に補正前の Transform を保存している。
- すべての変更は未コミット。`git commit` / `git push` / 外部アップロードは行っていない。

---

## 付録 A. 主要コマンドと証拠ファイル

### A.1 パッケージ

```bash
cargo xtask package --release
```

証拠: `target/phase4_bundle/evidence/package.log`

### A.2 起動

```bash
open target/bundle/NeoUtl.app
env -i HOME="$HOME" PATH="/usr/bin:/bin:/usr/sbin:/sbin" TMPDIR="$TMPDIR" \
  ./target/bundle/NeoUtl.app/Contents/MacOS/NeoUtl
```

証拠:

- `target/phase4_bundle/evidence/open_launch.log`
- `target/phase4_bundle/evidence/clean_env_launch.log`

### A.3 書き出し

```bash
/opt/homebrew/bin/ffprobe -v error \
  -show_entries format=duration,size,bit_rate \
  -show_entries stream=codec_name,codec_type,width,height,r_frame_rate,nb_frames \
  -of default=noprint_wrappers=1 \
  target/phase4_bundle/exports/export_phase4_text_720p_final3.mp4
```

証拠:

- `target/phase4_bundle/evidence/export_720p.log`
- `target/phase4_bundle/evidence/export_runtime.log`
- `target/phase4_bundle/exports/phase4_text_frame_4.5s.png`

### A.4 署名・dylib 検査

```bash
codesign --verify --deep --strict --verbose=2 target/bundle/NeoUtl.app
codesign -dv --verbose=4 target/bundle/NeoUtl.app
spctl --assess --type execute -vv target/bundle/NeoUtl.app
```

証拠:

- `target/phase4_bundle/evidence/macho_files.txt`
- `target/phase4_bundle/evidence/macho_load_refs.txt`

### A.5 回帰

```bash
cargo fmt --all -- --check
cargo xtask build --release
cargo test --workspace --exclude NeoUtl
```

証拠:

- `target/phase4_bundle/evidence/regression_build.log`
- `target/phase4_bundle/evidence/regression_tests.log`

### A.6 証拠ファイル一覧

```text
target/phase4_bundle/
├── bundle_manifest.txt          # `find NeoUtl.app -print` の全 170 エントリ
└── evidence/
    ├── clean_env_launch.log
    ├── export_720p.log
    ├── export_runtime.log
    ├── gatekeeper_outcome.log
    ├── gatekeeper_prompt_ax.log
    ├── git_status.txt
    ├── macho_files.txt
    ├── macho_load_refs.txt
    ├── open_launch.log
    ├── package.log
    ├── phase4_diff_stat.txt
    ├── phase4_working_tree.diff
    ├── preflight.txt
    ├── regression_build.log
    └── regression_tests.log
```

---

## 付録 B. パッケージングログ抜粋

```text
[package] 1/7: 実行バイナリを配置
[package] 2/7: プラグインとリソースを配置
[package]   Resources/objects/ (23 files)
[package]   Resources/effects/ (69 files)
[package]   Resources/decoders/ (2 files)
[package]   Resources/scripts/ (0 files)
[package]   Resources/easings/ (0 files)
[package] 3/7: Homebrew 依存 dylib を再帰収集
[package]   収集した dylib: 19 種
[package] 4/7: install_name / rpath を書き換え
[package]   Homebrew 絶対パス残存 0 件（検査対象 Mach-O 55 件）
[package] 5/7: Info.plist / PkgInfo / アイコンを生成
[package]   LSMinimumSystemVersion=27.0（バンドル内 Mach-O の最大 minos）
[package]   CFBundleShortVersionString=0.8.11 / CFBundleVersion=1178
[package] 6/7: ad-hoc 署名（内側から外側）
[package]   ad-hoc 署名検証成功 (codesign --verify --deep --strict)
[package] 7/7: DMG / ZIP を生成
[package]   NeoUtl.app: 89.97 MB
[package]   DMG:        39.94 MB
[package]   ZIP:        35.03 MB
```

---

**Phase 4 判定: 完了（Pass）**  
ただし、動画プロジェクトの書き出しハングと、Developer ID 署名・公証・Hardened Runtime は Phase 5 の必須対応項目として残る。現時点の `.app` / `.dmg` / `.zip` は ad-hoc 署名のローカル検証用成果物であり、一般配布用ではない。
