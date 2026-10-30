//! 应用入口：先初始化日志，再执行业务逻辑。
//!
//! 日志由 `ngy-utils-tracing` 统一配置，输出形态随应用模式变化：
//! - `development`：控制台（pretty，带文件名/行号/线程），详尽到单个步骤
//! - `test`：控制台 + 文件（JSON）
//! - `production`：文件（JSON，按天轮转），只保留必要记录

use std::process::ExitCode;
use std::time::Instant;

use ngy_utils_tracing::{InitOptions, init};
use tracing::{debug, error, info};

/// 环境变量前缀：模式读取 `NGY_APP_MODE`，日志目录读取 `NGY_LOG_DIR`，其余同理。
///
/// 前缀隔离可以避免误读其他程序使用的同名变量；`RUST_LOG` 是唯一不带前缀的例外。
const ENV_PREFIX: &str = "NGY_";

fn main() -> ExitCode {
    let started = Instant::now();

    // `InitResult` 持有文件日志的 WorkerGuard 和后台清理任务，必须存活到进程结束才不丢日志。
    let logging = match init(
        InitOptions::new(ENV_PREFIX)
            // 调试构建下的默认级别：项目自身 crate 用 debug，其余依赖回退到 info，可被 RUST_LOG 覆盖。
            // `CARGO_CRATE_NAME` 已经是 tracing 使用的 target 名（连字符转下划线）。
            .crates(vec![env!("CARGO_CRATE_NAME").to_string()]),
    ) {
        Ok(logging) => logging,
        // 此时订阅者尚未建立，无法用结构化日志记录，只能写 stderr 以免污染 stdout。
        Err(error) => {
            eprintln!("日志初始化失败: {error:#}");
            return ExitCode::FAILURE;
        }
    };

    info!(
        mode = %logging.mode,
        version = env!("CARGO_PKG_VERSION"),
        "应用启动"
    );

    // 错误只在这一层记录一次：内部各层负责返回错误，不自行打点，避免重复日志。
    let exit_code = match run() {
        Ok(()) => {
            info!(
                result = "ok",
                elapsed_ms = started.elapsed().as_millis() as u64,
                "应用退出"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            error!(
                error = %error,
                result = "error",
                elapsed_ms = started.elapsed().as_millis() as u64,
                "应用异常退出"
            );
            ExitCode::FAILURE
        }
    };

    // 显式释放，确保文件日志在进程退出前完成 flush。
    drop(logging);
    exit_code
}

/// 业务逻辑入口。
///
/// 出错时返回错误而不是自行记录，由 [`main`] 在边界处统一记录一次。
fn run() -> anyhow::Result<()> {
    info!("执行业务逻辑");
    debug!(
        step = 1,
        "调试信息仅在 development 模式或 RUST_LOG 显式开启时可见"
    );
    Ok(())
}
