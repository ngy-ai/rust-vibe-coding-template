//! `app_core::execute` 的微基准（宪法 V）。
//!
//! 复现命令：
//!
//! ```bash
//! cargo bench -p app-core
//! ```
//!
//! 预算：单次调用 p95 < 1ms。用的是 criterion 默认统计，比较两次测量时看置信区间是否重叠，
//! 重叠即视为无差异，不去追平均值的小数点。
//!
//! 三个用例对应三条真实路径：常见短输入、上限长度输入、超长拒绝。拒绝路径走的是
//! `else` 分支，容易被后续改动悄悄挪到校验之后，单独跑一遍能立刻看出它的开销变化。

use app_core::{MAX_INPUT_CHARS, execute};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn execute_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("core::execute");

    for (label, input) in [
        ("short-64", "a".repeat(64)),
        ("max-4096", "b".repeat(MAX_INPUT_CHARS)),
    ] {
        group.bench_with_input(BenchmarkId::from_parameter(label), &input, |b, input| {
            b.iter(|| execute(black_box(input)));
        });
    }

    group.finish();

    let too_long = "c".repeat(MAX_INPUT_CHARS + 1);
    c.bench_function("core::execute/too-long", |b| {
        b.iter(|| execute(black_box(&too_long)));
    });
}

criterion_group!(benches, execute_benchmark);
criterion_main!(benches);
