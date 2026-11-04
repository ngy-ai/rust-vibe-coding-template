# Tasks: 共享核心的 HTTP 服务端

**Input**: `/specs/001-shared-core-server/`（[spec.md](./spec.md) · [plan.md](./plan.md)）

**Status**: 全部完成。本文件于 2026-11-04 回溯补写，记录实际落地顺序；勾选项代表已合入 `main`。

**Organization**: 按 `spec.md` 的用户故事分组，每条故事独立可测。

## Phase 1: Setup

- [x] T001 在 workspace 新增 `crates/server`，同时声明 `lib`（暴露 `router()`）与 `bin`（进程边界）
- [x] T002 声明依赖：axum 0.8、tokio 1、serde 1、anyhow 1、ngy-utils-tracing（详见 `crates/server/Cargo.toml`）
- [x] T003 复用 workspace 的 rustfmt 与 clippy 配置，不新增工具链

## Phase 2: Foundational

- [x] T004 确认 `app-core::execute` 与 `CoreError` 已提供所需契约，服务端不再包装校验
- [x] T005 在 `crates/server/src/dto.rs` 定义 `ExecuteRequest`（无默认值）与 `ExecuteResponse`
- [x] T006 在 `crates/server/src/error.rs` 定义 `ApiError`、固定错误体与两个 `From` 实现（`CoreError`、`JsonRejection`）
- [x] T007 约定错误只记录一次：核心层只返回，`http.rs` 在被翻译成响应时打点

**Checkpoint**: 错误结构固定，可以开始写路由

---

## Phase 3: User Story 1 - 远程调用方复用共享校验 (P1) 🎯 MVP

**Goal**: `POST /api/execute` 成功返回 200 与同源 `message`，非法输入返回 422 与结构化错误体。

**Independent Test**: `cargo test -p app-server`，用 `oneshot` 直接打路由，不绑定端口。

- [x] T008 [P] [US1] 在 `crates/server/tests/http_test.rs` 先写 5 条契约测试（成功、空白、超长、缺字段、语法错误）并看到它们失败
- [x] T009 [US1] 在 `crates/server/src/http.rs` 实现 `router()` 与 handler，挂载 `POST /api/execute`
- [x] T010 [P] [US1] 在 `crates/server/src/args.rs` 提供 `--addr`，默认 `127.0.0.1:8080`
- [x] T011 [US1] 在 `crates/server/src/main.rs` 初始化日志并映射退出码
- [x] T012 [US1] 测试转绿：`cargo test -p app-server`

**Checkpoint**: 单端点可用，契约由测试锁定

---

## Phase 4: User Story 2 - 运维者安全地起停服务 (P2)

**Goal**: 默认只监听回环地址；端口占用给出可执行错误；Ctrl+C 优雅退出且不丢日志。

**Independent Test**: 默认启动确认监听 `127.0.0.1:8080`；占用端口后启动确认错误含修正方向。

- [x] T013 [US2] `serve()` 的绑定失败错误包含地址与「换端口或用 --addr」
- [x] T014 [US2] 实现 `shutdown_signal()`，收到中断后停止接受新连接并在退出前 flush 文件日志
- [x] T015 [US2] 确认 production 模式 stdout 与 stderr 均无日志

**Checkpoint**: 服务可安全部署

---

## Phase 5: Polish

- [x] T016 补 `lib.rs` 的文档与再导出，说明本 crate 只做翻译不做校验
- [x] T017 本地验证：`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test --all-targets --all-features`
- [x] T018 CI 覆盖：格式、lint、测试、治理扫描、机密扫描、依赖审计全部通过

## Dependencies

- T008 依赖 T005–T007；T009 依赖 T008；T011 依赖 T009、T010。
- US2 依赖 US1 的 handler 存在，但可独立验证（只看进程行为）。
