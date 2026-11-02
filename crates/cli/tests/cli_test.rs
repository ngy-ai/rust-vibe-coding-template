use std::process::{Command, Output};

/// 启动真实二进制。
///
/// 工作目录切到系统临时目录：`production` 模式的日志文件按 cwd 解析，留在包根会
/// 污染工作区。不注入模式变量，因为 `.env` 经 `dotenvy` 的 override 语义优先于
/// 进程环境，所以契约测试只断言与日志模式无关的部分——stdout 与退出码。
fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_app-cli"))
        .args(args)
        .current_dir(std::env::temp_dir())
        .env_remove("RUST_LOG")
        .output()
        .expect("CLI 二进制应当可以启动")
}

#[test]
fn prints_result_to_stdout() {
    let output = run_cli(&["--input", "hello"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "已处理 5 个字符"
    );
}

#[test]
fn blank_input_fails_with_exit_code_1() {
    let output = run_cli(&["--input", "   "]);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "错误不得写入 stdout，业务数据通道必须保持干净"
    );
}
