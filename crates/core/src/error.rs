/// 共享逻辑的错误。
///
/// 每个变体都要说明输入哪里不合法、期望是什么（宪法 IV），各端再把它翻译成
/// 自己的形态：CLI 退出码、HTTP 状态码、移动端 FFI 异常。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    /// 输入去掉首尾空白后为空。
    #[error("输入不能为空：请提供非空白文本，例如 \"hello\"")]
    EmptyInput,
    /// 输入字符数超过上限。
    #[error("输入包含 {len} 个字符，超过上限 {max}：请截断后重试")]
    TooLong { len: usize, max: usize },
}
