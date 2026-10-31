/// 环境变量前缀：模式读取 `NGY_APP_MODE`，日志目录读取 `NGY_LOG_DIR`，其余同理。
///
/// 前缀隔离可以避免误读其他程序使用的同名变量；`RUST_LOG` 是唯一不带前缀的例外。
pub(crate) const ENV_PREFIX: &str = "NGY_";
