//! 服务端库：把共享核心暴露成 HTTP 接口。
//!
//! 路由、请求体与错误结构集中在这里；`main` 只负责解析地址、初始化日志并监听端口。
//! 契约测试直接对 [`router`] 发请求，不绑定套接字，因此可以在 CI 里稳定运行。

mod dto;
mod error;
mod http;

pub use http::{router, serve};
