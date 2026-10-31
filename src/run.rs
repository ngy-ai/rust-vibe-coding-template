use tracing::{debug, info};

/// 业务逻辑入口。
///
/// 出错时返回错误而不是自行记录，由 [`main`] 在边界处统一记录一次。
pub(crate) fn run() -> anyhow::Result<()> {
    info!("执行业务逻辑");
    debug!(
        step = 1,
        "调试信息仅在 development 模式或 RUST_LOG 显式开启时可见"
    );
    Ok(())
}
