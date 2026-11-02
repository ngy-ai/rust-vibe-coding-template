/// 初始化 FRB 的默认工具（panic hook、日志转发等）。
///
/// 由生成的 `RustLib.init()` 调用，Dart 侧不需要手动调用本函数。
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
