//! 桌面端入口：gpui-kit 打开窗口，业务逻辑全部来自 `app-core`。
//!
//! 与 CLI 的区别只在接触面：窗口没有 stdout，日志同样由 `ngy-utils-tracing`
//! 写入文件，界面自己展示成功与失败。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui_kit::{AppContext, WindowOptions};
use ngy_utils_tracing::{InitOptions, init};
use tracing::error;

mod ui;

fn main() {
    // `InitResult` 持有文件日志的 WorkerGuard，必须存活到窗口关闭之后才不丢日志。
    let logging = match init(
        InitOptions::new(app_core::ENV_PREFIX).crates(vec![env!("CARGO_CRATE_NAME").to_string()]),
    ) {
        Ok(logging) => logging,
        Err(error) => {
            eprintln!("日志初始化失败: {error:#}");
            return;
        }
    };

    gpui_kit::application().run(|cx| {
        // `init` 必须在打开窗口之前调用，且每个进程只调用一次。
        gpui_kit::init(cx);

        let opened =
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(ui::Root::new));

        match opened {
            Ok(_) => {}
            Err(error) => error!(error = %error, "打开主窗口失败"),
        }
    });

    drop(logging);
}
