# Implementation Plan: 共享核心的 HTTP 服务端

**Branch**: `refactor/workspace-split` | **Date**: 2026-11-02 | **Spec**: 无独立 spec，随三端拆分一并落地

## Summary

把 `app-core` 暴露成 `POST /api/execute`，让 PC 端与移动端之外的调用方复用同一套输入校验。
服务端不新增业务规则：空白与 4096 字符上限仍由 `app-core::execute` 判定。

## Technical Context

**Language/Version**: Rust 1.97，edition 2024

**Primary Dependencies**: axum 0.8、tokio 1、clap 4、serde 1、ngy-utils-tracing 0.7

**Storage**: 无

**Testing**: `cargo test -p app-server`；用 `tower::ServiceExt::oneshot` 直接打路由，不绑定端口

**Target Platform**: Linux 服务器（CI 的 ubuntu-latest 覆盖）

**Project Type**: web-service

**Performance Goals**: 沿用宪法 V 的库默认预算，`app-core::execute` p95 < 1ms；HTTP 端到端本期不设门禁

**Constraints**: 默认只绑 `127.0.0.1:8080`，对外暴露需显式 `--addr`；stdout 不含日志

**Scale/Scope**: 单端点，无状态，无持久化

## Constitution Check

| 原则 | 结论 |
| --- | --- |
| I 文件规模 | 新增源文件均不超过 120 行 |
| II 质量 | 无死代码；`ApiError` 的两个 `From` 各有当前使用者 |
| III 测试 | 5 条契约测试覆盖成功、两类业务拒绝、缺字段、语法错误 |
| IV 接触面 | 错误体固定 `{error:{kind,message}}`；`message` 与 CLI 输出同源 |
| V 性能 | 无循环内 IO、无二次复杂度、无基准变更 |
| VI 简单性 | 见 Complexity Tracking |
| VII 可观测性 | 进入/退出带 `operation`、`result`、`elapsed_us`；错误只在边界记录一次 |
| VIII 安全 | 输入上限统一在 core 校验；无 SQL/命令拼接；不记录请求体 |

## Project Structure

```text
crates/server/
├── src/
│   ├── main.rs      # 入口：地址、日志、退出码
│   ├── args.rs      # --addr
│   ├── lib.rs       # 文档与再导出
│   ├── http.rs      # router / serve / handler
│   ├── dto.rs       # 请求与响应类型
│   └── error.rs     # ApiError 与统一错误体
└── tests/
    └── http_test.rs
```

**Structure Decision**: 与 `core` / `cli` 一致——lib 暴露 `router()` 供测试复用，bin 只处理进程边界。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 第四个单元 `crates/server`（宪法 VI 的初始实现上限是三个） | PC 端与移动端之外的调用方需要以 HTTP 复用同一套校验；服务端的进程边界（端口监听、并发连接、状态码）与 CLI 完全不同 | 并进 `app-cli`：会让 CLI 拉入 tokio/axum，拖累宪法 V 的 p95 < 100ms 启动预算，且 `cargo run` 行为随子命令漂移；并进 `app-core`：核心层必须保持零平台依赖，否则移动端 FFI 复用会被 HTTP 运行时污染 |
