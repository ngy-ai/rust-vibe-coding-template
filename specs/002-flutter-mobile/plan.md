# Implementation Plan: Flutter + flutter_rust_bridge 手机端

**Branch**: `refactor/workspace-split` | **Date**: 2026-11-02 | **Spec**: 无独立 spec，随三端拆分落地

## Summary

手机端从原来的「uniffi 生成 Kotlin/Swift 绑定」改为 **Flutter + flutter_rust_bridge 2.13**：
Dart 侧统一一套 UI 代码同时出 Android 与 iOS，Rust 侧只保留一个把 `app-core` 转成
Dart 函数的薄绑定层。业务规则、错误文案仍全部来自 `app-core`。

## Technical Context

**Language/Version**: Rust 1.97（edition 2024）、Dart 3.11 / Flutter stable

**Primary Dependencies**: flutter_rust_bridge 2.13.0（Rust 与 Dart 版本必须一致）、
`flutter_rust_bridge_codegen` 2.13.0（仅开发机需要）

**Storage**: 无

**Testing**: `cargo test -p app_mobile`（无逻辑，仅保证编译）、
`flutter test`（widget 冒烟，不触发 FFI）、`flutter analyze`

**Target Platform**: Android（Windows/macOS 均可交叉编译）、iOS（仅 macOS 可出包）

**Project Type**: mobile-app + library（绑定层）

**Performance Goals**: 沿用宪法 V 的库默认预算；`execute` 为同步调用，不引入 isolate 往返

**Constraints**: Dart 绑定的 `flutter_rust_bridge` 版本与 Rust crate 必须严格一致（`=2.13.0`）；
生成物不得手改

**Scale/Scope**: 单屏、单接口

## Constitution Check

| 原则 | 结论 |
| --- | --- |
| I 文件规模 | `frb_generated.rs` 头部带 `@generated`，258 行；手写文件均 < 60 行 |
| II 质量 | 绑定层无死代码；生成物不参与人工评审 |
| III 测试 | Rust 侧无业务逻辑（只转调 core），Dart 侧有 widget 冒烟测试 |
| IV 接触面 | Dart 抛出的异常消息与 CLI/HTTP 同源 |
| V 性能 | 同步调用，无额外线程与拷贝；无基准变更 |
| VI 简单性 | 见 Complexity Tracking |
| VII 可观测性 | 日志由 Rust 侧 tracing 承担；FRB 默认转发 panic |
| VIII 安全 | 输入上限仍在 core 校验；无凭据入库 |

## Project Structure

```text
crates/mobile/                # Rust 绑定层
├── Cargo.toml                # cdylib + staticlib + lib
└── src/
    ├── lib.rs
    ├── api.rs                # 暴露给 Dart 的模块清单
    ├── api/execute.rs        # 唯一的业务导出
    ├── api/init.rs           # #[frb(init)]
    └── frb_generated.rs      # 生成物，勿手改

apps/mobile/                  # Flutter 工程
├── flutter_rust_bridge.yaml  # rust_input / rust_root / dart_output
├── lib/main.dart
├── lib/src/rust/             # 生成的 Dart 绑定
├── rust_builder/             # cargokit 打包插件（生成物）
├── android/  ios/
└── test/widget_test.dart
```

**Structure Decision**: Rust 侧留在 Cargo workspace（`crates/mobile`），Flutter 侧放在
`apps/mobile`；两者通过 `flutter_rust_bridge.yaml` 的 `rust_root` 相对路径连接，
Rust 代码不再单独维护第二份 `Cargo.lock`。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 第五个单元 `crates/mobile`（宪法 VI 的初始上限是三个，serve 已占第四个） | Flutter 需要 `cdylib`/`staticlib` 产物与 `#[frb]` 标注，这些都不该污染 `app-core`；绑定层必须能在 Android/iOS 目标上单独编译 | 直接给 `app-core` 加 `#[frb]` 标注与 crate-type：会让核心库依赖 FRB 宏与 codegen，桌面端、CLI、服务端被动继承一套跨语言工具链；且 `cargo test -p app-core` 会被 FFI 目标类型牵制 |
| `apps/mobile` 不在 Cargo workspace 内 | Flutter 工程必须有自己的 `pubspec.yaml` 与平台目录，放进 `crates/` 会破坏「crates 只放 Rust crate」的约定 | 把 Dart 代码塞进 `crates/mobile`：`flutter` 工具链以 pubspec 为根，混放会导致 `flutter pub get` 与 `cargo` 互相扫描对方目录 |
