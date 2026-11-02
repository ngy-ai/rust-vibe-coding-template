//! 移动端绑定层：把 `app-core` 暴露给 Flutter。
//!
//! 这一层只做三件事：把 Dart 传来的值交给核心、把核心的错误转成可抛出的异常、
//! 启动 FRB 的默认工具。业务规则一律留在 `app-core`，这里不复制任何校验。
//!
//! Dart 侧代码由 `flutter_rust_bridge_codegen generate` 生成到
//! `apps/mobile/lib/src/rust/`，配置文件在 `apps/mobile/flutter_rust_bridge.yaml`。

pub mod api;
mod frb_generated;
