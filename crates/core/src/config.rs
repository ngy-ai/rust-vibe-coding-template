/// 本应用所有可配置环境变量的统一前缀。
///
/// 模式读取 `NGY_APP_MODE`，日志目录读取 `NGY_LOG_DIR`；`RUST_LOG` 是唯一
/// 不带前缀的例外。各端共用同一份前缀，避免同一个配置在不同端读到不同的名字。
pub const ENV_PREFIX: &str = "NGY_";
