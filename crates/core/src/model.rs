/// 单次调用允许的最大字符数，按 Unicode 标量计数而不是字节。
///
/// 上限存在的意义是让三端共用同一个边界：CLI 不需要为超长参数单独限流，
/// 服务端据此返回 422，移动端据此提示截断。
pub const MAX_INPUT_CHARS: usize = 4096;

/// 共享逻辑的成功结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// 给用户看的一行结果：CLI 写入 stdout，桌面窗口渲染成文本。
    pub message: String,
}
