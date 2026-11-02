//! 三端共享的业务逻辑。
//!
//! 这一层不依赖 GUI、HTTP 或平台相关 crate：桌面端、服务端与移动端都通过它
//! 执行同一套输入校验与业务规则。错误以 [`CoreError`] 返回，由各端翻译成自己的
//! 形态（CLI 退出码、HTTP 状态码、FFI 异常），本层不记录错误日志。

mod config;
mod error;
mod execute;
mod model;

pub use config::ENV_PREFIX;
pub use error::CoreError;
pub use execute::execute;
pub use model::{MAX_INPUT_CHARS, Output};
