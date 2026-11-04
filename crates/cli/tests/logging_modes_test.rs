//! 日志模式的契约测试（宪法 VII）。
//!
//! 三种模式对 stdout / stderr / 文件的约定不同，这里各起一个真实进程验证：
//!
//! - `production`：只写文件，stdout 与 stderr 都保持洁净；
//! - `development`：控制台写 stderr，stdout 仍只有业务数据；
//! - `test`：控制台与 JSON 文件双写，stdout 仍只有业务数据。
//!
//! 每个用例都用独立的临时目录，避免日志文件互相覆盖，也不污染工作区。

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 一条成功调用在任一模式下都必须产出的业务数据。
const EXPECTED_STDOUT: &str = "已处理 5 个字符";

/// 退出码为 0 时返回 stdout，否则带上 stderr 一起 panic，便于定位。
fn stdout_of(output: &Output) -> String {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// 在 `mode` 下运行 CLI，日志目录指向一个全新的临时目录，返回输出与日志目录。
fn run_in_mode(mode: &str, case: &str) -> (Output, PathBuf) {
    let log_dir = std::env::temp_dir().join(format!("app-cli-logging-{mode}-{case}"));
    let _ = std::fs::remove_dir_all(&log_dir);

    let output = Command::new(env!("CARGO_BIN_EXE_app-cli"))
        .args(["--input", "hello"])
        // 工作目录切到临时目录：`.env` 由 dotenvy 的 override 语义优先于进程环境，
        // 留在仓库根会让测试读到开发机的配置。
        .current_dir(std::env::temp_dir())
        .env("NGY_APP_MODE", mode)
        .env("NGY_LOG_DIR", &log_dir)
        .env_remove("RUST_LOG")
        .output()
        .expect("CLI 二进制应当可以启动");

    (output, log_dir)
}

/// 日志目录里的第一个文件；没有文件时返回 `None`。
fn first_log_file(dir: &Path) -> Option<(String, String)> {
    let entry = std::fs::read_dir(dir).ok()?.flatten().next()?;
    let name = entry.file_name().to_string_lossy().into_owned();
    let body = std::fs::read_to_string(entry.path()).unwrap_or_default();
    Some((name, body))
}

#[test]
fn production_writes_only_to_file() {
    let (output, log_dir) = run_in_mode("production", "file-only");

    assert_eq!(stdout_of(&output), EXPECTED_STDOUT);
    assert!(
        output.stderr.is_empty(),
        "production 模式不得写 stderr，实际: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let (name, body) = first_log_file(&log_dir)
        .unwrap_or_else(|| panic!("production 模式应当写日志文件，目录: {}", log_dir.display()));
    assert!(
        name.starts_with("app"),
        "日志文件名应当带前缀与日期，实际: {name}"
    );
    // JSON 文件层：字段名是结构化日志的最低要求（宪法 VII）。
    assert!(
        body.contains("\"operation\""),
        "日志缺少 operation 字段: {body}"
    );
    assert!(body.contains("\"result\""), "日志缺少 result 字段: {body}");
}

#[test]
fn development_logs_to_stderr_only() {
    let (output, log_dir) = run_in_mode("development", "console");

    assert_eq!(stdout_of(&output), EXPECTED_STDOUT);
    assert!(
        !output.stderr.is_empty(),
        "development 模式应当往 stderr 输出日志"
    );
    // development 只有控制台层，不落文件。
    assert!(
        first_log_file(&log_dir).is_none(),
        "development 模式不应写日志文件，目录: {}",
        log_dir.display()
    );
}

#[test]
fn test_mode_writes_console_and_file() {
    let (output, log_dir) = run_in_mode("test", "dual");

    assert_eq!(stdout_of(&output), EXPECTED_STDOUT);
    assert!(
        !output.stderr.is_empty(),
        "test 模式应当往 stderr 输出控制台日志"
    );
    assert!(
        first_log_file(&log_dir).is_some(),
        "test 模式应当同时写日志文件，目录: {}",
        log_dir.display()
    );
}
