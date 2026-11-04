# Tasks: Flutter + flutter_rust_bridge 手机端

**Input**: `/specs/002-flutter-mobile/`（[spec.md](./spec.md) · [plan.md](./plan.md)）

**Status**: 全部完成。本文件于 2026-11-04 回溯补写，记录实际落地顺序；勾选项代表已合入 `main`。

**Organization**: 按 `spec.md` 的用户故事分组，每条故事独立可测。

## Phase 1: Setup

- [x] T001 新建 `crates/mobile`，`crate-type` 同时声明 `cdylib`、`staticlib` 与 `lib`
- [x] T002 声明依赖：`flutter_rust_bridge =2.13.0`（与 Dart 侧严格一致）、`app-core`、anyhow
- [x] T003 新建 `apps/mobile` Flutter 工程与 `flutter_rust_bridge.yaml`（`rust_input` / `rust_root` / `dart_output`）

## Phase 2: Foundational

- [x] T004 在 `crates/mobile/src/api/execute.rs` 用 `#[frb(sync)]` 导出 `execute`，只转调 `app-core`
- [x] T005 在 `crates/mobile/src/api/init.rs` 提供 `#[frb(init)]` 初始化入口
- [x] T006 运行 `flutter_rust_bridge_codegen generate`，产出 `frb_generated.rs` 与 `lib/src/rust/`
- [x] T007 接入 `rust_builder/`（cargokit），让 Android 与 iOS 构建时自动编译 Rust

**Checkpoint**: Dart 能调用 `execute`，其余 UI 可并行

---

## Phase 3: User Story 1 - 在手机上运行共享逻辑 (P1) 🎯 MVP

**Goal**: 单屏输入 → 运行 → 结果卡片；非法输入显示核心给出的错误文案。

**Independent Test**: 模拟器输入 `hello` 显示「已处理 5 个字符」；输入空白转为错误态。

- [x] T008 [P] [US1] `apps/mobile/lib/main.dart` 实现单屏界面：输入框、运行按钮、结果卡片
- [x] T009 [US1] 捕获 `AnyhowException` 并把核心错误文案渲染为错误态卡片
- [x] T010 [P] [US1] 写 `apps/mobile/test/widget_test.dart` 冒烟测试，不触发 FFI
- [x] T011 [US1] `flutter analyze` 清零后 `flutter test` 通过

**Checkpoint**: 手机端可用，错误路径不崩溃

---

## Phase 4: User Story 2 - 三端错误话术同源 (P2)

**Goal**: 同一非法输入在 Dart、CLI、HTTP 三处的提示文字一致。

**Independent Test**: 对空白输入分别经三端执行，比较得到的消息。

- [x] T012 [US2] 确认 Dart 侧不复制任何校验与文案，全部经由 `app-core::execute` 的 `CoreError`
- [x] T013 [US2] 在 `crates/mobile/src/api/execute.rs` 与 UI 处注明文案来源，避免后续本地化时各自改写

**Checkpoint**: 三端共用一套排错话术

---

## Phase 5: User Story 3 - 绑定不漂移 (P3)

**Goal**: 改 Rust 接口后忘记 codegen，CI 必然失败。

**Independent Test**: 只改 `crates/mobile/src/api/*.rs` 的签名、不 codegen，mobile job 因 `git diff` 非空而红。

- [x] T014 [US3] CI 的 mobile job：`pub get` → codegen → 校验 `apps/mobile/lib/src/rust` 与 `frb_generated.rs` 无差异 → `flutter analyze` → `flutter test`
- [x] T015 [P] [US3] CI 的 android job：JDK 17 + NDK，构建 debug APK 覆盖全 ABI
- [x] T016 [P] [US3] CI 的 ios job：`flutter build ios --debug --no-codesign`（签名属发布侧配置，不进 CI）

**Checkpoint**: 生成物与源码在 CI 上强制一致

---

## Phase 6: Polish

- [x] T017 固定 Flutter 版本（3.41.4）与代码生成器版本（2.13.0），避免工具漂移造成假 diff
- [x] T018 在 `AGENTS.md` 与 `README.md` 补手机端命令（`flutter analyze`、`flutter test`、codegen）

## Dependencies

- T006 依赖 T004、T005；T008 依赖 T006。
- T014 依赖 T006 的产物路径稳定；T015、T016 与 T014 可并行。
