## 1.1 Bundled native libraries

MacUtl.app bundles the following native libraries under
`Contents/Frameworks/`. All of them are unmodified Homebrew bottles built for
Apple Silicon (arm64) on 2026-09-18, except where noted. The exact versions are
the ones recorded by Homebrew at package time.

| Component | Version | License | Source |
|---|---:|---|---|
| FFmpeg (libavcodec/libavformat/libavfilter/libavdevice/libavutil/libswscale/libswresample) | 9.0.2 | **GPL-3.0-or-later**（`--enable-gpl --enable-version3`） | https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz |
| x264 | r3222 | GPL-2.0-or-later | https://code.videolan.org/videolan/x264 |
| x265 | 4.3 | GPL-2.0-or-later | https://bitbucket.org/multicoreware/x265_git |
| SVT-AV1 | 4.2.0 | BSD-3-Clause | https://gitlab.com/AOMediaCodec/SVT-AV1 |
| dav1d | 1.5.4 | BSD-2-Clause | https://code.videolan.org/videolan/dav1d |
| libvpx | 1.17.0 | BSD-3-Clause | https://chromium.googlesource.com/webm/libvpx |
| libvmaf | 3.2.1 | BSD-2-Clause-Patent | https://github.com/Netflix/vmaf |
| OpenSSL (libssl/libcrypto) | 3.6.4 | Apache-2.0 | https://github.com/openssl/openssl |
| LAME (libmp3lame) | 4.0 | LGPL-2.0-or-later | https://sourceforge.net/projects/lame/ |
| mpg123 | 1.33.7 | LGPL-2.1-only | https://www.mpg123.de/ |
| Opus | 1.6.1 | BSD-3-Clause | https://opus-codec.org/ |
| XZ / liblzma | 5.8.4 | 0BSD（liblzma） | https://tukaani.org/xz/ |

License texts are stored under `third_party/licenses/` in the repository and
are copied into `MacUtl.app/Contents/Resources/third_party/licenses/`.

## 1.2 FFmpeg licensing determination

The bundled FFmpeg was built by Homebrew with the following configure
arguments (measured with `ffmpeg -buildconf` on the build machine):

```text
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

Determination:

- `--enable-gpl` activates GPL-only FFmpeg components.
- `--enable-version3` selects version 3 of the GPL.
- Therefore the bundled FFmpeg is **GPL-3.0-or-later**, not LGPL.
- `--enable-nonfree` is **not** present, so the binary is redistributable.
- GPLv3 and AGPLv3 are compatible: GPLv3 section 13 expressly permits
  combining GPLv3 code with AGPLv3 code. MacUtl as a whole is distributed
  under AGPL-3.0-or-later; FFmpeg and the other GPL components retain their
  own licenses and copyright notices.
- x264 and x265 are GPL-2.0-or-later. They are used under GPLv3 terms inside
  the GPLv3 FFmpeg build. This is compatible with AGPL-3.0-or-later.
- mpg123 is LGPL-2.1-only. LGPLv2.1 section 3 permits re-licensing under
  GPLv2 or, at the recipient's option, a later GPL version; MacUtl relies on
  GPLv3 compatibility for this component. No mpg123 source was modified.
- OpenSSL is Apache-2.0, which is compatible with GPLv3/AGPLv3.

### 1.2.1 Corresponding source

MacUtl's complete source, build scripts and modification notices are published
at <https://github.com/Ren9618/MacUtl>. The bundled FFmpeg is an unmodified
Homebrew build; its corresponding source is the upstream release above
(SHA-256: `8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e`).
The Homebrew build recipe is
<https://github.com/Homebrew/homebrew-core/blob/master/Formula/f/ffmpeg.rb>.
If you need a copy of any third-party source from us, open an issue at
<https://github.com/Ren9618/MacUtl/issues>; we will provide it.

## 1.3 Independent JPEG Group credit

FFmpeg contains code derived from the Independent JPEG Group (IJG):
`libavcodec/jfdctfst.c`, `libavcodec/jfdctint_template.c` and
`libavcodec/jrevdct.c`. In accordance with the FFmpeg license notice:

> This software is based in part on the work of the Independent JPEG Group.

Those files are unmodified in the bundled Homebrew build.

## 1.4 Other bundled non-code assets

| Asset | Origin | License |
|---|---|---|
| App icon (`MacUtl.icns`, generated from `assets/neoutl/icon.svg`) | NeoUtl branding submodule, commit `0ab51ecdc00569cdff0170fd58f50c3a688d09bf` | CC0-1.0 |
| Japanese/English UI strings (`Contents/Resources/{objects,effects}/**/*.yml` and built-in `i18n` files) | NeoUtl locales submodule, commit `f546a55359dc18c3f6378977527a242029d3171d` | CC0-1.0 |
| Project/plugin SDK headers shipped in the source tree (`sdk/`) | NeoUtl SDK submodule, commit `dfe019b722577f5249981d259f12aae4884966cd` | See `sdk/` in the repository |
| Wiki content (`wiki/`) | NeoUtl wiki submodule, commit `143381e06262bb06985d07f61a8439596f58041c` | See `wiki/` in the repository |

The branding and locale assets are explicitly dedicated to the public domain
under CC0 1.0, so the upstream icon and translations may be redistributed.
