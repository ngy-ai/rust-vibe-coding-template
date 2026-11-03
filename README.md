# rust-vibe-coding-template

用自然语言驱动 AI 开发 Rust 项目的骨架：工具链、格式化、lint、测试与治理门禁已预置，
开发规范以 [`.specify/memory/constitution.md`](.specify/memory/constitution.md) 为准。

## 环境要求

- [rustup](https://rustup.rs/)；Rust 版本由 `rust-toolchain.toml` 固定为 1.97，
  rustfmt 与 clippy 随工具链安装
- 本地想跑依赖检查时装 `cargo install cargo-machete --locked`（CI 会自动装）

## 快速开始

```bash
cargo run                        # CLI（默认成员），默认 --input hello
cargo run -p app-server          # 服务端，默认 127.0.0.1:8080
cargo run -p app-desktop         # 桌面端，gpui-kit 窗口
cd apps/mobile && flutter run    # 手机端，需要 Flutter SDK 与 Android NDK
```

服务端只有一个端点，契约与 CLI 共用同一份文案：

```bash
curl -s localhost:8080/api/execute \
  -H 'content-type: application/json' \
  -d '{"input":"hello"}'
# {"message":"已处理 5 个字符"}
```

错误体固定为 `{"error":{"kind":"...","message":"..."}}`：`kind` 给程序判断，
`message` 与 CLI 的报错同源，空白输入与超长输入都返回 422。

### 手机端（Flutter + flutter_rust_bridge）

`apps/mobile` 是 Flutter 工程，`crates/mobile` 是 Rust 绑定层，两者由
`apps/mobile/flutter_rust_bridge.yaml` 连接。

Dart 绑定是生成物，**改完 `crates/mobile` 的导出函数后必须重新生成，否则 Dart 侧会和 Rust 侧漂移**：
生成代码仍引用旧函数名，`flutter analyze` 看不出来，要到运行时才报错。

```bash
cargo install flutter_rust_bridge_codegen --version 2.13.0 --locked   # 与两端版本严格一致
cd apps/mobile
flutter_rust_bridge_codegen generate
```

生成结果落在 `apps/mobile/lib/src/rust/`，不要手改；它与 Rust 接口属于同一次变更，
必须一起提交。Android 构建由 cargokit 调 cargo，需要 Android SDK/NDK；iOS 只能在 macOS 上出包。

## 目录结构

| 路径 | 作用 |
| --- | --- |
| `crates/core/` | 三端共享的业务逻辑与错误类型；不依赖 GUI、HTTP 与平台 API |
| `crates/cli/` | 命令行客户端（clap）：负责参数、stdout/stderr 分工与退出码 |
| `crates/server/` | 服务端（axum）：`POST /api/execute`，错误体与 CLI 文案同源 |
| `crates/mobile/` | 移动端绑定层（flutter_rust_bridge）：只转调 `app-core` |
| `crates/desktop/` | 桌面客户端（gpui-kit）：窗口只做展示与调用 `app-core` |
| `apps/mobile/` | Flutter 工程：UI、`rust_builder` 打包插件与生成的 Dart 绑定 |
| `.specify/memory/constitution.md` | 项目宪法，八条原则的唯一定义处 |
| `AGENTS.md` | 给编码代理的速查入口（索引，不含细则） |
| `.specify/templates/` | spec-kit 的 spec / plan / tasks / checklist 模板 |
| `.specify/scripts/` | spec-kit 的 PowerShell 与 Python 辅助脚本 |
| `.codebuddy/commands/` | `/speckit.*` 命令定义 |
| `.github/workflows/ci.yml` | CI：格式、lint + 依赖、测试、宪法治理扫描 |
| `rust-toolchain.toml` | 固定工具链与组件 |
| `rustfmt.toml` | `edition` / `style_edition` = 2024 |
| `Cargo.toml` | workspace 清单：成员、`default-members`、共享依赖与 MSRV |

## 质量门禁

与 `ci.yml` 的 job 一一对应：

| 检查 | 对应原则 |
| --- | --- |
| `cargo fmt --check` | II |
| `cargo clippy --all-targets --all-features -- -D warnings` | II |
| `cargo machete`（未使用依赖） | II |
| `cargo test --all-targets --all-features` | III |
| `cargo clippy -p app-desktop`（独立 Windows job） | II |
| `flutter analyze` + `flutter test` + 绑定漂移检查（独立 mobile job） | III |
| `flutter build apk --debug`（独立 android job，含 NDK 交叉编译） | III |
| `cargo audit`（依赖漏洞） | VIII |
| 机密扫描（私钥块、云密钥、平台令牌、误提交的 `.env`） | VIII |
| TODO 归属扫描 + `#[allow(dead_code)]` 拦截 | II |

`cargo fmt` 覆盖全部成员；`clippy` / `test` 在根目录只作用于 `default-members`
（`core`、`cli`、`server`），桌面端依赖图形栈，由单独的 Windows job 校验。

本地一次跑完：

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo machete
```

## 项目规范

宪法八条原则：

| 编号 | 原则 | 要点 |
| --- | --- | --- |
| I | 源码文件规模与拆分 | 单文件 700 行硬上限、600 行预警线，先拆再改 |
| II | 代码质量与可维护性 | 公共接口写契约、不留死代码、TODO 必须带 issue |
| III | 测试先行开发 | 先写测试并看到它失败，再用最少实现转绿 |
| IV | 对外接触面一致性 | CLI / 库 API / 错误消息统一，错误必须可执行 |
| V | 性能与效率 | 默认预算（p95 门禁 / p99 观察）、禁止二次复杂度与逐项 IO、变更须附测量、回归 >10% 需理由 |
| VI | 简单性与 YAGNI | 默认单 crate、不预留配置与推测性抽象、三行重复优于过早抽象 |
| VII | 可观测性与结构化日志 | 结构化字段、错误只记一次、stdout 放数据 / stderr 放日志、不记敏感信息 |
| VIII | 安全基线 | 边界校验输入、禁止拼接注入、机密不入库、依赖扫漏洞 |

规则细节只在宪法里维护，本文件不复述，避免两处漂移。

## spec-kit 工作流

典型顺序：

1. `/speckit.constitution` —— 修订治理原则（可选）
2. `/speckit.specify` —— 写清要做什么，产出 `specs/<feature>/spec.md`
3. `/speckit.clarify` —— 消除规格里的歧义
4. `/speckit.plan` —— 技术方案，产出 `plan.md`，含 Constitution Check
5. `/speckit.tasks` —— 拆成可执行任务
6. `/speckit.implement` —— 执行

辅助命令：`/speckit.analyze`、`/speckit.checklist`、`/speckit.taskstoissues`、
`/speckit.converge`。

## 作为模板使用

1. 改 `crates/*/Cargo.toml` 的 `name` 与 `[workspace.dependencies]` 里的路径别名，
   crate 名会自动变成下划线形式
2. 更新 `.github/CODEOWNERS`，把所有者换成你自己或团队
3. 第一次 `/speckit.plan` 时确认宪法 V 的默认预算是否沿用，未确认即视为沿用
4. 按需调整 `ci.yml` 的工具链版本与 job
5. 配置分支保护（见下），否则「变更必须走 PR」只是口头约定
6. 重写本 README

### 分支保护（只能在 GitHub 上手动开启）

仓库里的任何文件都管不住这条，只有分支保护能让宪法「拉取请求」一节真正生效：

1. Settings → Branches → Add branch protection rule，目标分支 `main`；
2. 勾选 **Require a pull request before merging**。单开发者仓库可把批准数设为 0，
   由 Governance 里的单开发者条款以自我证明兜底；
3. 勾选 **Require status checks to pass**，并选中全部九个：
   `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test`、`desktop (windows)`、
   `mobile (flutter)`、`android (build apk)`、`Constitution governance scan`、
   `cargo audit`、`Secret scan`；
4. 勾选 **Do not allow bypassing the above settings**，否则管理员仍能直接推送。

开启之前，直接 `git push origin main` 是会成功的——本模板自身的提交历史就是例子。

## 容易踩的约定

- 测试镜像源码：`crates/<crate>/src/<module>/` 对应 `crates/<crate>/tests/<module>_test.rs`
- TODO 只能写成 `TODO(#<issue>): 说明`；裸 `TODO:`、`TODO(TBD)` 会被 CI 拦下
- 死代码豁免只接受 `#[expect(dead_code, reason = "<理由>，见 #<issue>")]`，
  `#[allow(dead_code)]` 会被 CI 拦下
- 未使用的依赖会让 `cargo machete` 失败，先删依赖再加代码，不要反过来
- 改 `crates/mobile` 的导出函数后，要在 `apps/mobile` 跑一次
  `flutter_rust_bridge_codegen generate`，并把 `lib/src/rust/` 的改动一起提交；
  漏了这步 `flutter analyze` 不会报错，只有运行时才会发现 Dart 还在调旧符号
- 基准工具（`criterion`、`hyperfine`）只在真正要测量时引入，且与第一个基准同时提交
