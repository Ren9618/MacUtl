#!/usr/bin/env python3
"""
MacUtl macOS .app / DMG / ZIP packager.

This script is called by `cargo xtask package` after xtask has built and staged
the release binaries. It deliberately does not use Homebrew at runtime: every
non-system dylib is copied into Contents/Frameworks and its install name is
rewritten to @rpath, so the resulting bundle only needs macOS system libraries.

Usage:
    cargo xtask package --release
    python3 scripts/package_macos.py [--debug]
"""

from __future__ import annotations

import os
import plistlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

HOMEBREW_PREFIXES = (
    "/opt/homebrew/",
    "/usr/local/Cellar/",
    "/usr/local/opt/",
    "/usr/local/lib/",
)

APP_NAME = "MacUtl"
BUNDLE_ID = "works.rayno.macutl"
OBJECT_COUNT = 9
EFFECT_COUNT = 23


def log(message: str) -> None:
    print(f"[package] {message}", flush=True)


def fail(message: str) -> "None":
    print(f"[package] エラー: {message}", file=sys.stderr, flush=True)
    sys.exit(1)


def run(
    cmd: list[str],
    *,
    check: bool = True,
    capture: bool = False,
    cwd: Path | None = None,
) -> str:
    result = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
        text=True,
    )
    if check and result.returncode != 0:
        detail = (result.stderr or result.stdout or "").strip()
        fail(f"コマンド失敗 ({result.returncode}): {' '.join(cmd)}\n{detail}")
    return (result.stdout or "") if capture else ""


def is_homebrew_path(path: str) -> bool:
    return path.startswith(HOMEBREW_PREFIXES)


def otool_deps(path: Path) -> list[str]:
    out = run(["otool", "-L", str(path)], capture=True)
    deps: list[str] = []
    for line in out.splitlines()[1:]:
        line = line.strip()
        if not line:
            continue
        dep = line.split(" (compatibility version", 1)[0].strip()
        if dep:
            deps.append(dep)
    return deps


def rpaths(path: Path) -> list[str]:
    out = run(["otool", "-l", str(path)], capture=True)
    found: list[str] = []
    lines = out.splitlines()
    for index, line in enumerate(lines):
        if "cmd LC_RPATH" in line:
            for follow in lines[index + 1 : index + 4]:
                follow = follow.strip()
                if follow.startswith("path "):
                    found.append(follow[5:].split(" (offset", 1)[0].strip())
                    break
    return found


def add_rpath(path: Path, value: str) -> None:
    if value in rpaths(path):
        return
    run(["install_name_tool", "-add_rpath", value, str(path)], capture=True)


def macho_min_os(path: Path) -> tuple[int, int] | None:
    out = run(["vtool", "-show-build", str(path)], capture=True)
    current_cmd = ""
    versions: list[tuple[int, int]] = []
    for raw in out.splitlines():
        line = raw.strip()
        if line.startswith("cmd "):
            current_cmd = line[4:].strip()
            continue
        value: str | None = None
        if line.startswith("minos "):
            value = line.split()[1]
        elif current_cmd == "LC_VERSION_MIN_MACOSX" and line.startswith("version "):
            value = line.split()[1]
        if value:
            parts = value.split(".")
            try:
                versions.append((int(parts[0]), int(parts[1]) if len(parts) > 1 else 0))
            except ValueError:
                pass
    return max(versions) if versions else None


def app_machos(app_dir: Path) -> list[Path]:
    binaries: list[Path] = []
    macos_dir = app_dir / "Contents" / "MacOS"
    if macos_dir.is_dir():
        for entry in sorted(macos_dir.iterdir()):
            if entry.is_file() and not entry.is_symlink():
                binaries.append(entry)
    resources_dir = app_dir / "Contents" / "Resources"
    if resources_dir.is_dir():
        for root, _, files in os.walk(resources_dir):
            for name in sorted(files):
                path = Path(root) / name
                if path.suffix == ".dylib" and not path.is_symlink():
                    binaries.append(path)
    frameworks_dir = app_dir / "Contents" / "Frameworks"
    if frameworks_dir.is_dir():
        for entry in sorted(frameworks_dir.iterdir()):
            if entry.is_file() and not entry.is_symlink():
                binaries.append(entry)
    return binaries


def collect_homebrew_dylibs(app_dir: Path, frameworks_dir: Path) -> dict[str, str]:
    """Copy all Homebrew dylibs referenced by bundled Mach-O files.

    Returns a mapping of original install-name basename -> copied file name.
    """
    seen_real: set[str] = set()
    aliases: dict[str, str] = {}
    queue: list[Path] = app_machos(app_dir)

    while queue:
        current = queue.pop(0)
        for dep in otool_deps(current):
            if not is_homebrew_path(dep):
                continue
            if not os.path.exists(dep):
                fail(f"依存 dylib が見つかりません: {current} -> {dep}")
            real = os.path.realpath(dep)
            real_name = os.path.basename(real)
            aliases[os.path.basename(dep)] = real_name
            if real in seen_real:
                continue
            seen_real.add(real)
            destination = frameworks_dir / real_name
            if not destination.exists():
                shutil.copy2(real, destination)
                os.chmod(destination, 0o755)
                log(f"  同梱: {dep} -> Frameworks/{real_name}")
            queue.append(destination)

    # Homebrew uses versioned real files behind unversioned symlinks. The
    # install names in the binaries generally use the unversioned name, so
    # recreate those aliases inside Frameworks.
    for alias, real_name in sorted(aliases.items()):
        if alias == real_name:
            continue
        link = frameworks_dir / alias
        if link.exists() or link.is_symlink():
            continue
        os.symlink(real_name, link)
        log(f"  別名: Frameworks/{alias} -> {real_name}")

    return aliases


def rewrite_install_names(app_dir: Path, frameworks_dir: Path) -> None:
    executable = app_dir / "Contents" / "MacOS" / APP_NAME
    host = app_dir / "Contents" / "MacOS" / "maolan-plugin-host"

    add_rpath(executable, "@executable_path/../Frameworks")
    if host.exists():
        add_rpath(host, "@executable_path/../Frameworks")

    # Frameworks: give every dylib a stable @rpath id and a loader-relative
    # search path, then point its Homebrew dependencies at @rpath.
    for entry in sorted(frameworks_dir.iterdir()):
        if entry.is_symlink() or not entry.is_file() or entry.suffix != ".dylib":
            continue
        add_rpath(entry, "@loader_path")
        run(["install_name_tool", "-id", f"@rpath/{entry.name}", str(entry)], capture=True)
        for dep in otool_deps(entry):
            if is_homebrew_path(dep):
                run(
                    ["install_name_tool", "-change", dep, f"@rpath/{os.path.basename(dep)}", str(entry)],
                    capture=True,
                )

    # Resources: plugin dylibs are dlopen()ed from Contents/Resources. They
    # normally only link libSystem, but handle any FFmpeg dependency too.
    resources_dir = app_dir / "Contents" / "Resources"
    for root, _, files in os.walk(resources_dir):
        for name in sorted(files):
            path = Path(root) / name
            if path.suffix != ".dylib" or path.is_symlink():
                continue
            add_rpath(path, "@executable_path/../Frameworks")
            run(["install_name_tool", "-id", f"@rpath/{path.name}", str(path)], capture=True)
            for dep in otool_deps(path):
                if is_homebrew_path(dep):
                    run(
                        ["install_name_tool", "-change", dep, f"@rpath/{os.path.basename(dep)}", str(path)],
                        capture=True,
                    )

    for binary in (executable, host):
        if not binary.exists():
            continue
        for dep in otool_deps(binary):
            if is_homebrew_path(dep):
                run(
                    ["install_name_tool", "-change", dep, f"@rpath/{os.path.basename(dep)}", str(binary)],
                    capture=True,
                )


def verify_no_homebrew(app_dir: Path) -> None:
    leftovers: list[tuple[Path, str]] = []
    for binary in app_machos(app_dir):
        for dep in otool_deps(binary):
            if is_homebrew_path(dep):
                leftovers.append((binary, dep))
    if leftovers:
        for binary, dep in leftovers:
            print(f"  {binary}: {dep}", file=sys.stderr)
        fail(f"Homebrew 絶対パスが {len(leftovers)} 件残っています")
    log(f"  Homebrew 絶対パス残存 0 件（検査対象 Mach-O {len(app_machos(app_dir))} 件）")


def minimum_system_version(app_dir: Path) -> str:
    versions: list[tuple[int, int]] = []
    for binary in app_machos(app_dir):
        version = macho_min_os(binary)
        if version:
            versions.append(version)
    if not versions:
        return "11.0"
    major, minor = max(versions)
    return f"{major}.{minor}"


def create_icon(workspace_root: Path, resources_dir: Path, bundle_dir: Path) -> Path:
    icon_svg = workspace_root / "assets" / "neoutl" / "icon.svg"
    if not icon_svg.is_file():
        fail(f"アイコン素材が見つかりません: {icon_svg}")

    iconset = bundle_dir / f"{APP_NAME}.iconset"
    if iconset.exists():
        shutil.rmtree(iconset)
    iconset.mkdir(parents=True)

    # Quick Look renders the SVG at its intrinsic 128x128 size. Wrap it with a
    # 1024x1024 viewport so the source artwork is rasterised at full size, then
    # let sips produce the standard iconset sizes.
    svg_text = icon_svg.read_text(encoding="utf-8")
    svg_1024 = svg_text.replace('width="128" height="128"', 'width="1024" height="1024"', 1)
    if svg_1024 == svg_text:
        fail("icon.svg の width/height を 1024 に変更できませんでした")
    temp_svg = bundle_dir / f"{APP_NAME}-icon-1024.svg"
    temp_svg.write_text(svg_1024, encoding="utf-8")

    render_dir = bundle_dir / f"{APP_NAME}-icon-render"
    if render_dir.exists():
        shutil.rmtree(render_dir)
    render_dir.mkdir()
    run(["qlmanage", "-t", "-s", "1024", "-o", str(render_dir), str(temp_svg)], capture=True)
    rendered = render_dir / f"{temp_svg.name}.png"
    if not rendered.is_file():
        fail(f"SVG の PNG 変換に失敗しました: {rendered}")

    sizes = [
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ]
    for size, name in sizes:
        run(["sips", "-z", str(size), str(size), str(rendered), "--out", str(iconset / name)], capture=True)

    icns_path = resources_dir / f"{APP_NAME}.icns"
    run(["iconutil", "-c", "icns", str(iconset), "-o", str(icns_path)], capture=True)
    shutil.rmtree(iconset)
    shutil.rmtree(render_dir)
    temp_svg.unlink(missing_ok=True)
    log(f"  アイコン生成: {icon_svg.relative_to(workspace_root)} -> Resources/{APP_NAME}.icns")
    return icns_path


def write_info_plist(
    contents_dir: Path,
    *,
    version: str,
    build_number: str,
    minimum_os: str,
) -> None:
    info = {
        "CFBundleDevelopmentRegion": "ja",
        "CFBundleDisplayName": APP_NAME,
        "CFBundleExecutable": APP_NAME,
        "CFBundleIconFile": f"{APP_NAME}.icns",
        "CFBundleIdentifier": BUNDLE_ID,
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": APP_NAME,
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": build_number,
        "LSApplicationCategoryType": "public.app-category.video",
        "LSMinimumSystemVersion": minimum_os,
        "NSHighResolutionCapable": True,
        "NSRequiresAquaSystemAppearance": False,
        "NSSupportsAutomaticGraphicsSwitching": True,
    }
    with open(contents_dir / "Info.plist", "wb") as handle:
        plistlib.dump(info, handle, sort_keys=True)
    (contents_dir / "PkgInfo").write_text("APPL????", encoding="ascii")
    run(["plutil", "-lint", str(contents_dir / "Info.plist")], capture=True)


def sign_bundle(app_dir: Path) -> None:
    frameworks_dir = app_dir / "Contents" / "Frameworks"
    resources_dir = app_dir / "Contents" / "Resources"
    macos_dir = app_dir / "Contents" / "MacOS"

    for entry in sorted(frameworks_dir.iterdir()):
        if not entry.is_symlink() and entry.is_file():
            run(["codesign", "--force", "--sign", "-", str(entry)], capture=True)
    for root, _, files in os.walk(resources_dir):
        for name in sorted(files):
            path = Path(root) / name
            if path.suffix == ".dylib" and not path.is_symlink():
                run(["codesign", "--force", "--sign", "-", str(path)], capture=True)
    for name in ("maolan-plugin-host", APP_NAME):
        path = macos_dir / name
        if path.exists():
            run(["codesign", "--force", "--sign", "-", str(path)], capture=True)
    run(["codesign", "--force", "--sign", "-", str(app_dir)], capture=True)
    run(["codesign", "--verify", "--deep", "--strict", str(app_dir)], capture=True)
    log("  ad-hoc 署名検証成功 (codesign --verify --deep --strict)")


def directory_size(path: Path) -> int:
    total = 0
    for root, _, files in os.walk(path):
        for name in files:
            file_path = Path(root) / name
            if not file_path.is_symlink():
                total += file_path.stat().st_size
    return total


def main() -> None:
    profile = "debug" if "--debug" in sys.argv else "release"
    workspace_root = Path(__file__).resolve().parent.parent
    target_dir = workspace_root / "target" / profile
    bundle_dir = workspace_root / "target" / "bundle"
    app_dir = bundle_dir / f"{APP_NAME}.app"
    contents_dir = app_dir / "Contents"
    macos_dir = contents_dir / "MacOS"
    resources_dir = contents_dir / "Resources"
    frameworks_dir = contents_dir / "Frameworks"

    neoutl_bin = target_dir / "NeoUtl"  # crate/binary name stays upstream-compatible
    host_bin = target_dir / "maolan-plugin-host"
    if not neoutl_bin.is_file():
        fail(f"バイナリが見つかりません: {neoutl_bin}（先に cargo xtask build --release）")
    if not host_bin.is_file():
        fail(f"maolan-plugin-host が見つかりません: {host_bin}")

    # Verify the staged plugin set before creating anything, so a stale or
    # partial build cannot produce a seemingly valid bundle.
    object_dir = target_dir / "objects"
    effect_dir = target_dir / "effects"
    object_count = len(list(object_dir.glob("*.dylib"))) if object_dir.is_dir() else 0
    effect_count = len(list(effect_dir.glob("*.dylib"))) if effect_dir.is_dir() else 0
    if object_count != OBJECT_COUNT:
        fail(f"objects プラグイン数が {object_count} 件です（期待値 {OBJECT_COUNT}）")
    if effect_count != EFFECT_COUNT:
        fail(f"effects プラグイン数が {effect_count} 件です（期待値 {EFFECT_COUNT}）")

    version = "0.8.11"
    cargo_toml = (workspace_root / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', cargo_toml, re.MULTILINE)
    if match:
        version = match.group(1)

    build_number = os.environ.get("NEOUTL_BUILD_NUMBER", "").strip()
    if not build_number:
        try:
            build_number = run(["git", "rev-list", "--count", "HEAD"], capture=True, cwd=workspace_root).strip()
        except SystemExit:
            build_number = ""
    if not build_number.isdigit():
        build_number = "1"

    log(f"=== {APP_NAME} macOS バンドル生成開始 ({profile}) ===")
    log(f"  出力先: {app_dir}")

    if app_dir.exists():
        shutil.rmtree(app_dir)
    for path in (macos_dir, resources_dir, frameworks_dir):
        path.mkdir(parents=True, exist_ok=True)

    log("1/7: 実行バイナリを配置")
    shutil.copy2(neoutl_bin, macos_dir / APP_NAME)
    shutil.copy2(host_bin, macos_dir / "maolan-plugin-host")
    os.chmod(macos_dir / APP_NAME, 0o755)
    os.chmod(macos_dir / "maolan-plugin-host", 0o755)

    log("2/7: プラグインとリソースを配置")
    for name in ("objects", "effects", "decoders", "scripts", "easings", "themes"):
        source = target_dir / name
        if source.is_dir():
            shutil.copytree(source, resources_dir / name)
            log(f"  Resources/{name}/ ({sum(1 for _ in source.rglob('*') if _.is_file())} files)")
        elif name in ("objects", "effects", "decoders", "easings"):
            fail(f"必須リソースディレクトリがありません: {source}")
        else:
            log(f"  スキップ: {source}（存在しません）")

    # AGPL compliance: ship the license, the fork's change notice, the audit
    # and the complete third-party notices inside the app bundle as well as in
    # the repository.
    log("2/7: ライセンス文書を同梱")
    for name in ("LICENSE", "THIRD_PARTY_NOTICES.md", "LICENSE_AUDIT.md", "MACOS_CHANGES.md"):
        source = workspace_root / name
        if not source.is_file():
            fail(f"ライセンス文書が見つかりません: {source}")
        shutil.copy2(source, resources_dir / name)
    license_source = workspace_root / "third_party" / "licenses"
    if not license_source.is_dir():
        fail(f"第三者ライセンス本文が見つかりません: {license_source}")
    shutil.copytree(license_source, resources_dir / "third_party" / "licenses")
    (resources_dir / "SOURCE.txt").write_text(
        "MacUtl corresponding source:\n"
        "https://github.com/Ren9618/MacUtl\n"
        "Tag: v0.8.11-mac.1 (base: NeoUtl 0.8.11, 0ead39350db55996810de53da79d02fbc98355f5)\n"
        "License: AGPL-3.0-or-later\n"
        "\n"
        "Bundled FFmpeg 9.0.2 is an unmodified Homebrew build (GPL-3.0-or-later).\n"
        "Source: https://ffmpeg.org/releases/ffmpeg-9.0.2.tar.xz\n"
        "SHA-256: 8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e\n",
        encoding="utf-8",
    )
    log(
        "  Resources: LICENSE, THIRD_PARTY_NOTICES.md, LICENSE_AUDIT.md, "
        "MACOS_CHANGES.md, SOURCE.txt, third_party/licenses/"
    )

    log("3/7: Homebrew 依存 dylib を再帰収集")
    aliases = collect_homebrew_dylibs(app_dir, frameworks_dir)
    if aliases:
        log(f"  収集した dylib: {len(aliases)} 種")
    else:
        log("  Homebrew 依存なし")

    log("4/7: install_name / rpath を書き換え")
    rewrite_install_names(app_dir, frameworks_dir)
    verify_no_homebrew(app_dir)

    log("5/7: Info.plist / PkgInfo / アイコンを生成")
    minimum_os = minimum_system_version(app_dir)
    write_info_plist(
        contents_dir,
        version=version,
        build_number=build_number,
        minimum_os=minimum_os,
    )
    create_icon(workspace_root, resources_dir, bundle_dir)
    log(f"  LSMinimumSystemVersion={minimum_os}（バンドル内 Mach-O の最大 minos）")
    log(f"  CFBundleShortVersionString={version} / CFBundleVersion={build_number}")

    log("6/7: ad-hoc 署名（内側から外側）")
    sign_bundle(app_dir)

    dmg_path = bundle_dir / f"{APP_NAME}-{version}-macos.dmg"
    zip_path = bundle_dir / f"{APP_NAME}-{version}-macos.zip"
    for path in (dmg_path, zip_path):
        path.unlink(missing_ok=True)

    log("7/7: DMG / ZIP を生成")
    # Add the usual "drag to Applications" affordance to the DMG. The app is
    # copied with ditto so the ad-hoc signature and resource forks survive.
    dmg_stage = bundle_dir / "dmg_stage"
    if dmg_stage.exists():
        shutil.rmtree(dmg_stage)
    dmg_stage.mkdir(parents=True)
    run(["ditto", str(app_dir), str(dmg_stage / f"{APP_NAME}.app")], capture=True)
    os.symlink("/Applications", dmg_stage / "Applications")
    run(
        [
            "hdiutil",
            "create",
            "-volname",
            APP_NAME,
            "-srcfolder",
            str(dmg_stage),
            "-ov",
            "-format",
            "UDZO",
            str(dmg_path),
        ],
        capture=True,
    )
    shutil.rmtree(dmg_stage)
    run(
        ["ditto", "-c", "-k", "--keepParent", "--norsrc", str(app_dir), str(zip_path)],
        capture=True,
    )

    app_bytes = directory_size(app_dir)
    dmg_bytes = dmg_path.stat().st_size
    zip_bytes = zip_path.stat().st_size
    log("=== 完了 ===")
    log(f"  {APP_NAME}.app: {app_bytes / (1024 * 1024):.2f} MB ({app_dir})")
    log(f"  DMG:        {dmg_bytes / (1024 * 1024):.2f} MB ({dmg_path})")
    log(f"  ZIP:        {zip_bytes / (1024 * 1024):.2f} MB ({zip_path})")


if __name__ == "__main__":
    main()
