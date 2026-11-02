use clap::Parser;

/// 命令行参数。
#[derive(Debug, Parser)]
#[command(version, about = "共享核心的终端客户端")]
pub(crate) struct Args {
    /// 交给共享核心处理的文本；空白输入会被拒绝。
    #[arg(short, long, default_value = "hello")]
    pub(crate) input: String,
}
