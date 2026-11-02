use std::io::Write;

use app_core::execute;
use tracing::debug;

use crate::args::Args;

/// 执行业务逻辑：调用共享核心，并把结果写到 stdout。
///
/// 出错时返回错误而不是自行记录，由入口在边界处统一记录一次。
pub(crate) fn run(args: Args) -> anyhow::Result<()> {
    debug!(chars = args.input.chars().count(), "读取到命令行输入");

    let output = execute(&args.input)?;

    // 用 `writeln!` 而不是 `println!`：管道被提前关闭时返回错误，而不是 panic。
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{}", output.message)?;

    Ok(())
}
