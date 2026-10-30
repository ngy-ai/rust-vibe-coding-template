//! 应用入口：先初始化日志，再执行业务逻辑。
//!
//! 日志由 `ngy-utils-tracing` 统一配置，输出形态随应用模式变化：
//! - `development`：控制台（pretty，带文件名/行号/线程）
//! - `test`：控制台 + 文件（JSON）
//! - `production`：文件（JSON，按天轮转）

use ngy_utils_tracing::{InitOptions, init};
use tracing::{debug, info};

/// 环境变量前缀：模式读取 `NGY_APP_MODE`，日志目录读取 `NGY_LOG_DIR`，其余同理。
///
/// 前缀隔离可以避免误读其他程序使用的同名变量；`RUST_LOG` 是唯一不带前缀的例外。
const ENV_PREFIX: &str = "NGY_";

fn main() -> anyhow::Result<()> {
    // `InitResult` 持有文件日志的 WorkerGuard 和后台清理任务，必须存活到进程结束才不丢日志。
    let logging = init(
        InitOptions::new(ENV_PREFIX)
            // 调试构建下的默认级别：项目自身 crate 用 debug，其余依赖回退到 info，可被 RUST_LOG 覆盖。
            // `CARGO_CRATE_NAME` 已经是 tracing 使用的 target 名（连字符转下划线）。
            .crates(vec![env!("CARGO_CRATE_NAME").to_string()]),
    )?;

    info!(
        mode = %logging.mode,
        version = env!("CARGO_PKG_VERSION"),
        "应用启动"
    );

    run();

    info!("应用退出");
    Ok(())
}

fn run() {
    info!("执行业务逻辑");
    debug!(
        step = 1,
        "调试信息仅在 development 模式或 RUST_LOG 显式开启时可见"
    );
}
