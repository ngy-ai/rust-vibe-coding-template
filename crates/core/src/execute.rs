use std::time::Instant;

use tracing::{debug, info};

use crate::error::CoreError;
use crate::model::{MAX_INPUT_CHARS, Output};

/// 执行三端共享的业务逻辑。
///
/// 输入在边界处一次校验完成（宪法 VIII）：去掉首尾空白后不得为空，字符数不得
/// 超过 [`MAX_INPUT_CHARS`]；业务逻辑内部不再做兜底校验。
///
/// # 错误
///
/// - [`CoreError::EmptyInput`]：输入只有空白字符。
/// - [`CoreError::TooLong`]：输入字符数超过上限。
///
/// 错误只返回、不记录：日志由调用方在各自的边界记录一次（宪法 VII）。
pub fn execute(input: &str) -> Result<Output, CoreError> {
    let started = Instant::now();
    let trimmed = input.trim();
    debug!(chars = trimmed.chars().count(), "共享逻辑开始");

    if trimmed.is_empty() {
        return Err(CoreError::EmptyInput);
    }

    let chars = trimmed.chars().count();
    if chars > MAX_INPUT_CHARS {
        return Err(CoreError::TooLong {
            len: chars,
            max: MAX_INPUT_CHARS,
        });
    }

    info!(
        operation = "core.execute",
        result = "ok",
        chars,
        elapsed_us = started.elapsed().as_micros() as u64,
        "共享逻辑完成"
    );

    Ok(Output {
        message: format!("已处理 {chars} 个字符"),
    })
}
