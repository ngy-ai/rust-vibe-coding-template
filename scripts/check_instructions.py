#!/usr/bin/env python3
"""用指令数预算做性能回归门禁（宪法 V）。

`cargo bench -p app-core --bench execute_iai` 输出每个基准的指令数，这个脚本把它与
`crates/core/benches/instructions.json` 里的预算比较：超过预算 × (1 + 容差) 即失败。

为什么用指令数而不是墙钟时间：共享 runner 的负载不可控，同一份代码的耗时能差出一倍，
按时间设门禁只会训练人忽略红色。valgrind 的指令数与机器无关，同一份二进制永远同一个数。

用法：

    python scripts/check_instructions.py             # CI 用，超预算即失败
    python scripts/check_instructions.py --update     # 重新记录预算（需在 PR 里说明理由）

容差默认 10%，与宪法 V 的回归线一致；测试噪声不该吃掉这条线。
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

BUDGET_FILE = Path("crates/core/benches/instructions.json")
BENCH_ARGS = ["cargo", "bench", "-p", "app-core", "--bench", "execute_iai"]

# iai-callgrind 的输出形如：
#   execute_short
#     Instructions:                1,234
BENCH_NAME = re.compile(r"^(execute_[a-z_]+)\s*$")
INSTRUCTIONS = re.compile(r"Instructions:\s*([\d,]+)")


def run_benchmarks() -> dict[str, int]:
    """跑一遍基准，返回 {基准名: 指令数}。"""
    print("$ " + " ".join(BENCH_ARGS), file=sys.stderr)
    completed = subprocess.run(
        BENCH_ARGS, capture_output=True, text=True, encoding="utf-8", errors="replace"
    )
    output = completed.stdout + completed.stderr
    # 基准本身崩了要立刻停下：把它当成「预算内」会把门禁静默变成装饰。
    if completed.returncode != 0:
        print(output, file=sys.stderr)
        raise SystemExit("基准运行失败，无法比较预算")

    measured: dict[str, int] = {}
    current: str | None = None
    for line in output.splitlines():
        stripped = line.strip()
        if match := BENCH_NAME.match(stripped):
            current = match.group(1)
            continue
        if current and (match := INSTRUCTIONS.search(stripped)):
            measured[current] = int(match.group(1).replace(",", ""))
            current = None

    if not measured:
        print(output, file=sys.stderr)
        raise SystemExit("没有解析到任何指令数：输出格式变了？")
    return measured


def main() -> int:
    parser = argparse.ArgumentParser(description="检查指令数预算")
    parser.add_argument("--update", action="store_true", help="用本次结果重写预算文件")
    parser.add_argument("--tolerance", type=float, default=0.10, help="允许的回归比例，默认 0.10")
    args = parser.parse_args()

    measured = run_benchmarks()

    if args.update:
        budgets = dict(sorted(measured.items()))
        BUDGET_FILE.write_text(json.dumps(budgets, indent=2) + "\n", encoding="utf-8")
        print(f"已写入 {BUDGET_FILE}：{budgets}")
        return 0

    if not BUDGET_FILE.exists():
        raise SystemExit(f"缺少预算文件 {BUDGET_FILE}；先跑 --update 生成并在 PR 里说明基线来源")
    budgets = json.loads(BUDGET_FILE.read_text(encoding="utf-8"))

    failed = False
    for name, actual in sorted(measured.items()):
        budget = budgets.get(name)
        if budget is None:
            print(f"✗ {name}: 没有预算记录（新增基准需要 --update 补齐）", file=sys.stderr)
            failed = True
            continue
        limit = int(budget * (1 + args.tolerance))
        mark = "✓" if actual <= limit else "✗"
        print(f"{mark} {name}: {actual} 指令（预算 {budget}，上限 {limit}）")
        if actual > limit:
            failed = True

    stale = sorted(set(budgets) - set(measured))
    if stale:
        print(f"✗ 预算里有已不存在的基准：{', '.join(stale)}（删掉或改名时同时更新预算）", file=sys.stderr)
        failed = True

    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
