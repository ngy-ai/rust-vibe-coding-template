use std::net::SocketAddr;

use clap::Parser;

/// 服务端命令行参数。
#[derive(Debug, Parser)]
#[command(version, about = "共享核心的 HTTP 服务端")]
pub(crate) struct Args {
    /// 监听地址；默认只绑本机，对外提供服务时需显式指定网卡地址。
    #[arg(long, default_value = "127.0.0.1:8080")]
    pub(crate) addr: SocketAddr,
}
