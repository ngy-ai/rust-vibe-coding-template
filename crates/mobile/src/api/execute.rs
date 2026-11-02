/// 执行共享核心，返回给用户看的一行结果。
///
/// 同步函数：单次调用远低于一帧预算，没必要让 Dart 侧多写一个 `await`。
///
/// # 错误
///
/// 输入为空白或超过 4096 字符时抛出异常。异常消息与 CLI 的错误输出、HTTP 的
/// `message` 字段同源，三端共用一套排错话术（宪法 IV）。
#[flutter_rust_bridge::frb(sync)]
pub fn execute(input: String) -> anyhow::Result<String> {
    app_core::execute(&input)
        .map(|output| output.message)
        .map_err(|error| anyhow::anyhow!(error))
}
