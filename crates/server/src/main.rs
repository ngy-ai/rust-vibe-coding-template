//! 服务端入口：解析监听地址、初始化日志、启动 HTTP 服务。
//!
//! 与 CLI 一致：日志由 `ngy-utils-tracing` 统一配置，错误只在这一层记录一次。
//! 业务规则全部来自 `app-core`，HTTP 层只负责解包参数与翻译错误。

use std::process::ExitCode;

use clap::Parser;
use ngy_utils_tracing::{InitOptions, init};
use tracing::{error, info};

mod args;

#[tokio::main]
async fn main() -> ExitCode {
    let args = args::Args::parse();

    // `InitResult` 持有文件日志的 WorkerGuard，必须在服务退出后才释放。
    let logging = match init(
        InitOptions::new(app_core::ENV_PREFIX).crates(vec![env!("CARGO_CRATE_NAME").to_string()]),
    ) {
        Ok(logging) => logging,
        Err(error) => {
            eprintln!("日志初始化失败: {error:#}");
            return ExitCode::FAILURE;
        }
    };

    let exit_code = match app_server::serve(args.addr).await {
        Ok(()) => {
            info!(result = "ok", "服务退出");
            ExitCode::SUCCESS
        }
        Err(error) => {
            error!(error = %error, result = "error", "服务异常退出");
            ExitCode::FAILURE
        }
    };

    drop(logging);
    exit_code
}
