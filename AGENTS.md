# AGENTS.md

Rust + spec-kit 的 vibe coding 模板。本文件是给编码代理的速查入口，**只做索引，不定义规则**：
细则一律以 [`.specify/memory/constitution.md`](.specify/memory/constitution.md) 为准，
冲突时以宪法为准。

## 目录地图

| 路径 | 放什么 |
| --- | --- |
| `crates/core/` | 三端共享的业务逻辑；不依赖 GUI、HTTP 与平台 API |
| `crates/cli/` | 命令行客户端，负责参数、stdout/stderr 分工与退出码 |
| `crates/server/` | 服务端（axum）；`POST /api/execute`，错误体与 CLI 文案同源 |
| `crates/mobile/` | 移动端绑定层（flutter_rust_bridge）；只转调 `app-core`，不含业务规则 |
| `crates/desktop/` | 桌面客户端（gpui-kit）；界面只展示与调用 `app-core` |
| `apps/mobile/` | Flutter 工程；`lib/src/rust/` 是生成物，改接口后要重新 codegen |
| `.specify/memory/constitution.md` | 八条原则与治理规则，唯一权威 |
| `.specify/templates/` | spec / plan / tasks / checklist 模板 |
| `.specify/scripts/` | spec-kit 辅助脚本（PowerShell / Python） |
| `.codebuddy/commands/` | `/speckit.*` 命令定义 |
| `specs/` | 每个 feature 的 spec、plan、tasks（由 `/speckit.specify` 创建） |
| `crates/core/benches/` | criterion 基准；`cargo bench -p app-core` |
| `scripts/init.py` | 复制模板后的一次性改名脚本（crate 名、bundle id、环境变量前缀） |
| `dist-workspace.toml` | cargo-dist 配置：可分发二进制与目标平台 |
| `.github/workflows/release.yml` | cargo-dist 生成的发布工作流（tag 触发，勿手改） |
| `.github/workflows/ci.yml` | 七类门禁、十二个 job；Linux job 固定 `ubuntu-24.04` |
| `.github/branch-protection.md` | 分支保护设置与十二个必需检查的记录、核对命令 |
| `.github/pull_request_template.md` | PR 模板：宪法自查、影响范围、例外与自我证明 |
| `.github/ISSUE_TEMPLATE/` | bug 与功能请求表单；空白 issue 已关闭 |
| `.github/SECURITY.md` | 漏洞报告走私密渠道，不开放公开 issue |
| `.github/dependabot.yml` | 三个生态的每周依赖更新分组策略 |
| `deny.toml` | 许可证白名单、禁用依赖与依赖来源规则 |
| `.github/CODEOWNERS` | 核心模块与所有者 |

## 命令

```bash
cargo run                           # CLI（default-members 之一）
cargo run -p app-server             # 服务端，默认 127.0.0.1:8080
cargo run -p app-desktop            # 桌面端，gpui-kit
cargo fmt --check
cd apps/mobile && flutter analyze && flutter test   # 手机端（需 Flutter SDK）
cd apps/mobile && flutter_rust_bridge_codegen generate  # 改完 Rust 接口后重生成绑定
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy -p app-desktop --all-targets -- -D warnings
cargo test --all-features   # 不带 --all-targets：iai 基准需要 runner，会被当测试跑失败
cargo machete     # 未使用依赖，需 cargo install cargo-machete --locked
cargo audit
dist plan                                  # 校验发布配置会产出什么（不会真的发）       # 依赖漏洞
cargo deny check  # 许可证、禁用依赖与依赖来源，需 cargo install cargo-deny --locked
cargo bench -p app-core  # 基准（criterion），复现命令与预算见 README「性能测量」
```

本地把前四条跑一遍，就等于复现了 CI 的格式、lint 与测试门禁。

## 硬约束（判据以宪法对应小节为准）

| 约束 | 一句话判据 | 详见 |
| --- | --- | --- |
| 文件规模 | 超过上限先拆再改，不堆着写 | I |
| 质量与可维护性 | 无死代码、无注释代码、TODO 必须带 issue | II |
| 测试先行 | 先写测试并看到它失败，再写最少实现 | III |
| 对外接触面 | CLI 与库 API 命名统一，错误必须可执行 | IV |
| 性能 | 涉及性能的变更必须附前后测量 | V |
| 简单性 | 没有当前消费者就不写配置、开关与抽象 | VI |
| 可观测性 | 结构化字段，错误只记一次，stdout 放数据 / stderr 放日志 | VII |
| 安全 | 输入在边界校验，禁止拼接注入，机密不入库 | VIII |

## 禁止事项

- 不手改 `Cargo.lock`，用 `cargo` 命令更新。
- 不改 `.specify/memory/constitution.md`，除非用户明确要求；改了按治理条款递增版本。
- 不为让 CI 变绿而添加 lint 抑制。死代码豁免只接受
  `#[expect(dead_code, reason = "...，见 #<issue>")]`。
- 不提交 `.env`、令牌或私钥。
- 不直接提交到 `main`：变更走 PR，分支保护开启后会被拦下。

## 冲突时

宪法 > 本文件 > 其他惯例。规格与宪法冲突时，先改宪法再改规格。
