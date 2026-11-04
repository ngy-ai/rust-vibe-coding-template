#!/usr/bin/env python3
"""把这份模板改造成你自己的项目。

复制模板后要改的东西散落在 Cargo 清单、Rust 标识符、Flutter 包名、Android/iOS bundle id
与 `NGY_` 环境变量前缀里，漏一处就编译不过或运行时读错配置。这个脚本按一张固定的替换表
一次改完，并把文件名/目录名里的旧包名一并重命名。

用法：

    python scripts/init.py --name my-app                 # 只打印计划，不写盘
    python scripts/init.py --name my-app --apply         # 真正写入
    python scripts/init.py --name my-app --env-prefix ACME_ --bundle-id com.acme.app --apply

默认值：环境变量前缀取大写的项目名加下划线（`my-app` → `MY_APP_`），
bundle id 取 `com.example.<项目名>_mobile`。

写盘后会跑一次 `cargo metadata` 校验清单自洽，并列出仍需人工处理的步骤。
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

# 不参与替换的目录：构建产物与依赖缓存里没有需要改的标识符。
SKIP_DIRS = {".git", "target", "build", ".dart_tool", "Pods", "node_modules", ".gradle"}

# 这些后缀按二进制处理，读取失败即跳过。
BINARY_SUFFIXES = {
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".ttf", ".otf",
    ".jar", ".keystore", ".jks", ".so", ".dll", ".dylib", ".a", ".exe",
}

# 模板里的 crate 后缀；`mobile` 的 Cargo 包名用下划线（`app_mobile`），其余用连字符。
HYPHEN_CRATES = ("core", "cli", "server", "desktop")
UNDERSCORE_CRATES = ("mobile",)

# Android 的 Kotlin 源码目录按包名分层，改名时要跟着 bundle id 走。
KOTLIN_DIR = Path("apps/mobile/android/app/src/main/kotlin")


def tracked_files(root: Path) -> list[Path]:
    """优先用 `git ls-files` 取清单，非 git 仓库时退回遍历。"""
    try:
        completed = subprocess.run(
            ["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True
        )
        names = [p for p in completed.stdout.decode("utf-8").split("\0") if p]
        if names:
            return [root / name for name in names]
    except (OSError, subprocess.CalledProcessError):
        pass

    files: list[Path] = []
    for path in root.rglob("*"):
        if any(part in SKIP_DIRS for part in path.parts) or not path.is_file():
            continue
        files.append(path)
    return files


def build_rules(name: str, env_prefix: str, bundle_id: str) -> list[tuple[str, str]]:
    """构造有序且去重的替换表：长 token 在前，避免短串先截断长串。"""
    crate = name.strip().lower()
    ident = crate.replace("-", "_")
    ordered: list[tuple[str, str]] = [
        # Flutter 的打包插件名比包名更长，必须先替换。
        ("rust_lib_app_mobile", f"rust_lib_{ident}_mobile"),
        ("app_mobile", f"{ident}_mobile"),
        # bundle id：iOS 用驼峰形态，Android 用下划线形态。
        ("com.example.appMobile", bundle_id),
        ("com.example.app_mobile", bundle_id),
    ]
    for suffix in HYPHEN_CRATES:
        ordered.append((f"app-{suffix}", f"{crate}-{suffix}"))
        ordered.append((f"app_{suffix}", f"{ident}_{suffix}"))
    for suffix in UNDERSCORE_CRATES:
        ordered.append((f"app_{suffix}", f"{ident}_{suffix}"))
    ordered.append(("NGY_", env_prefix))

    # 同一个 token 可能被上面的分支重复加进来，保留第一次出现的位置。
    seen: set[str] = set()
    rules: list[tuple[str, str]] = []
    for old, new in ordered:
        if old in seen:
            continue
        seen.add(old)
        rules.append((old, new))
    return rules


def rewrite(root: Path, files: list[Path], rules: list[tuple[str, str]], apply: bool) -> dict[str, int]:
    """按规则替换文件内容，返回每条规则的命中文件数。"""
    hits = {old: 0 for old, _ in rules}
    for path in files:
        if path.suffix.lower() in BINARY_SUFFIXES:
            continue
        if any(part in SKIP_DIRS for part in path.relative_to(root).parts):
            continue
        try:
            original = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue

        updated = original
        for old, new in rules:
            if old in updated:
                hits[old] += 1
                updated = updated.replace(old, new)
        if updated != original and apply:
            path.write_text(updated, encoding="utf-8")
    return hits


def rename_paths(root: Path, rules: list[tuple[str, str]], apply: bool) -> list[tuple[Path, Path]]:
    """重命名文件名与目录名中的旧 token；目录按深度倒序处理，避免父目录先改导致路径失效。"""
    kotlin_parts = KOTLIN_DIR.parts
    renames: list[tuple[Path, Path]] = []
    for path in sorted(root.rglob("*"), key=lambda p: len(p.parts), reverse=True):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        # Android 的包目录要跟着 bundle id 走，由 move_kotlin_package 统一处理，
        # 这里跳过，否则计划里会出现「先改成 com/example/<新名>、再搬到 <bundle id>」两条。
        if path.relative_to(root).parts[: len(kotlin_parts)] == kotlin_parts:
            continue
        name = path.name
        new_name = name
        for old, new in rules:
            new_name = new_name.replace(old, new)
        if new_name == name:
            continue
        target = path.with_name(new_name)
        renames.append((path, target))
        if apply:
            path.rename(target)
    return renames


def move_kotlin_package(root: Path, bundle_id: str, apply: bool) -> list[tuple[Path, Path]]:
    """把 Android 的 Kotlin 源码目录搬到与 bundle id 一致的分层路径下。"""
    kotlin_root = root / KOTLIN_DIR
    if not kotlin_root.is_dir():
        return []

    package_dir = kotlin_root.joinpath(*bundle_id.split("."))
    sources = [path for path in kotlin_root.rglob("*.kt")]
    if not sources or package_dir.is_dir():
        return []

    source_dir = sources[0].parent
    if source_dir == package_dir:
        return []

    moves = [(source_dir, package_dir)]
    if apply:
        package_dir.parent.mkdir(parents=True, exist_ok=True)
        source_dir.rename(package_dir)
        # 清掉搬走后留下的空目录，直到 kotlin 根为止。
        parent = source_dir.parent
        while parent != kotlin_root and parent.is_dir() and not any(parent.iterdir()):
            parent.rmdir()
            parent = parent.parent
    return moves


def verify(root: Path, name: str) -> bool:
    """用 `cargo metadata` 确认 workspace 清单改名后仍然自洽。"""
    crate = name.strip().lower()
    ident = crate.replace("-", "_")
    expected = {f"{crate}-{suffix}" for suffix in HYPHEN_CRATES} | {
        f"{ident}_{suffix}" for suffix in UNDERSCORE_CRATES
    }
    try:
        completed = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=root,
            capture_output=True,
            check=True,
            encoding="utf-8",
            errors="replace",
        )
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"! cargo metadata 失败：{error}", file=sys.stderr)
        return False

    packages = {package["name"] for package in json.loads(completed.stdout)["packages"]}
    missing = sorted(expected - packages)
    if missing:
        print(f"! 改名后的 workspace 缺少这些包：{', '.join(missing)}", file=sys.stderr)
        return False
    return True


def main() -> int:
    # Windows 控制台默认 GBK，直接打印中文会在部分终端变成乱码。
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass

    parser = argparse.ArgumentParser(description="把模板改造成你自己的项目")
    parser.add_argument("--name", required=True, help="项目名，例如 my-app；crate 会变成 my-app-core")
    parser.add_argument("--env-prefix", help="环境变量前缀，默认由项目名推导，例如 MY_APP_")
    parser.add_argument("--bundle-id", help="Android/iOS 包标识，默认 com.example.<项目名>_mobile")
    parser.add_argument("--apply", action="store_true", help="真正写盘；不加时只打印计划")
    parser.add_argument("--root", default=".", help="仓库根目录，默认当前目录")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    crate = args.name.strip().lower()
    if not crate or not crate.replace("-", "").replace("_", "").isalnum():
        parser.error("--name 只能包含字母、数字、连字符与下划线")

    ident = crate.replace("-", "_")
    env_prefix = args.env_prefix or f"{ident.upper()}_"
    bundle_id = args.bundle_id or f"com.example.{ident}_mobile"

    rules = build_rules(args.name, env_prefix, bundle_id)
    files = tracked_files(root)

    hits = rewrite(root, files, rules, apply=args.apply)
    renames = rename_paths(root, rules, apply=args.apply)
    moves = move_kotlin_package(root, bundle_id, apply=args.apply)

    mode = "已写入" if args.apply else "计划（未写盘，加 --apply 执行）"
    print(f"== {mode} ==")
    print(f"项目名: {crate} / 标识符前缀: {ident} / 环境变量前缀: {env_prefix} / bundle id: {bundle_id}")
    print("\n内容替换：")
    for old, new in rules:
        if hits[old]:
            print(f"  {old} -> {new}：{hits[old]} 个文件")
    print("\n重命名：")
    for path, target in renames + moves:
        print(f"  {path.relative_to(root)} -> {target.relative_to(root)}")
    if not renames and not moves:
        print("  无")

    if not args.apply:
        return 0

    ok = verify(root, args.name)
    print("\n仍需人工处理：")
    print("  1. 重写 README.md 的标题与简介")
    print("  2. 更新 .github/CODEOWNERS 的所有者")
    print("  3. 在 GitHub 上配置分支保护（见 .github/branch-protection.md）")
    print("  4. 跑 cargo test、flutter pub get && flutter analyze、flutter_rust_bridge_codegen generate")
    print("  5. iOS 真机发布前，在 Xcode 里设置签名团队")
    print("  6. 把 crates/cli 与 crates/server 的 repository 字段改成你的仓库（cargo-dist 用它拼下载链接）")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
