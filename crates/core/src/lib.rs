//! 三端共享的业务逻辑。
//!
//! 这一层不依赖 GUI、HTTP 或平台相关 crate，各端通过它读取同一份配置与同一套规则。

mod config;

pub use config::ENV_PREFIX;
