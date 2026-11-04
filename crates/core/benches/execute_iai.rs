//! `app_core::execute` 的指令数基准（宪法 V 的回归门禁）。
//!
//! 与 `benches/execute.rs` 的分工：criterion 给人看（墙钟时间、可以对比本地两次改动），
//! 这个文件给 CI 看——valgrind 统计的指令数与机器负载无关，同一份代码永远得到同一个数，
//! 因此可以拿绝对预算做门禁，不会因为共享 runner 抖动而误报。
//!
//! 复现命令（需要 valgrind 与同版本的 iai-callgrind-runner）：
//!
//! ```bash
//! cargo bench -p app-core --bench execute_iai
//! python scripts/check_instructions.py
//! ```
//!
//! 注意：基准函数上不要写 `///` 文档注释——`library_benchmark` 会把它们当作 `#[doc]`
//! 属性传给宏展开，结果是 `Invalid attribute: 'doc'`。用普通注释即可。

use std::hint::black_box;

use app_core::{MAX_INPUT_CHARS, execute};
use iai_callgrind::{library_benchmark, library_benchmark_group, main};

fn short_input() -> String {
    "a".repeat(64)
}

fn max_input() -> String {
    "b".repeat(MAX_INPUT_CHARS)
}

fn over_limit_input() -> String {
    "c".repeat(MAX_INPUT_CHARS + 1)
}

// 常见短输入：走成功路径的最小开销。
#[library_benchmark]
#[bench::short(setup = short_input)]
fn execute_short(input: String) -> usize {
    black_box(
        execute(black_box(&input))
            .expect("短输入应当成功")
            .message
            .len(),
    )
}

// 上限输入：字符计数与拼接的最坏情况。
#[library_benchmark]
#[bench::max(setup = max_input)]
fn execute_max(input: String) -> usize {
    black_box(
        execute(black_box(&input))
            .expect("上限输入应当成功")
            .message
            .len(),
    )
}

// 超长拒绝：确认拒绝分支没有被挪到校验之后。
#[library_benchmark]
#[bench::too_long(setup = over_limit_input)]
fn execute_too_long(input: String) -> usize {
    black_box(
        execute(black_box(&input))
            .expect_err("超长必须被拒绝")
            .to_string()
            .len(),
    )
}

library_benchmark_group!(
    name = execute;
    benchmarks = execute_short, execute_max, execute_too_long
);

main!(library_benchmark_groups = execute);
