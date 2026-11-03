# Feature Specification: Flutter + flutter_rust_bridge 手机端

**Feature Branch**: `refactor/workspace-split`

**Created**: 2026-11-02

**Status**: 已实现（`spec.md` 于 2026-11-03 回溯补写，描述上线行为；实现计划见 [plan.md](./plan.md)）

**Input**: 用一套 Dart 代码同时出 Android 与 iOS，业务逻辑与错误文案全部来自 `app-core`

## User Scenarios & Testing

### User Story 1 - 在手机上运行共享逻辑 (Priority: P1)

用户在单屏界面输入一段文本并点击运行，界面显示共享核心返回的一行结果；输入不合法时显示错误提示，
而不是崩溃或空白。

**Why this priority**: 这是手机端存在的唯一理由；没有可运行的界面，FFI 绑定无法被验证。

**Independent Test**: 在模拟器上输入 `hello`，确认结果卡片显示「已处理 5 个字符」；输入仅空白，
确认卡片转为错误态并显示核心给出的文案。

**Acceptance Scenarios**:

1. **Given** 应用已启动，**When** 输入 `hello` 并运行，**Then** 绿色结果卡片显示「已处理 5 个字符」
2. **Given** 输入只有空格，**When** 运行，**Then** 红色错误卡片显示「输入不能为空……」并给出示例
3. **Given** 输入 4097 个字符，**When** 运行，**Then** 错误卡片显示实际字符数与 4096 上限

---

### User Story 2 - 三端错误话术同源 (Priority: P2)

用户从 CLI 换到手机端时，同一类错误的提示文字不变，只需记住一套排错话术。

**Why this priority**: 各端各自措辞会让「同一份业务规则」在用户眼里变成三套规则，也会让契约测试失效。

**Independent Test**: 对同一非法输入，比较 Dart 侧捕获的异常消息、CLI 的 stderr 输出与 HTTP 的
`error.message`，三者逐字相同。

**Acceptance Scenarios**:

1. **Given** 输入为空白，**When** 分别经 CLI、HTTP、手机端执行，**Then** 三者给出的提示文字一致

---

### User Story 3 - 绑定不漂移 (Priority: P3)

改动 Rust 导出接口后忘记重新生成绑定，CI 会拦下，而不是等到运行时抛出找不到符号。

**Why this priority**: 生成物与源码不一致是最常见的移动端故障，且本地难以发现。

**Independent Test**: 故意修改 `crates/mobile/src/api/*.rs` 的签名但不 codegen，确认 CI 的
mobile job 因 `git diff` 非空而失败。

**Acceptance Scenarios**:

1. **Given** Rust 接口已改但未重新生成绑定，**When** 触发 CI，**Then** mobile job 失败并指出生成物有差异

---

### Edge Cases

- 首帧之前调用绑定：必须先 `await RustLib.init()`，否则原生库未加载。
- 中文输入：按 Unicode 标量计数，4096 个汉字不算超限。
- 键盘回车提交与按钮点击必须走同一条路径，避免两处逻辑分叉。
- 结果与错误共用一张卡片，只切换配色与图标，不新增第二套布局。

## Requirements

### Functional Requirements

- **FR-001**: 应用启动时 MUST 先完成 `RustLib.init()`，之后才允许调用任何绑定函数。
- **FR-002**: 界面 MUST 提供单行输入、运行入口与结果展示区；回车与按钮触发同一段逻辑。
- **FR-003**: 成功时 MUST 以成功态展示核心返回的 `message`；失败时 MUST 以错误态展示异常消息原文。
- **FR-004**: 业务校验、边界值与文案 MUST 全部来自 `app-core`，Dart 侧不得复制判断逻辑。
- **FR-005**: `execute` MUST 以同步方式导出，不引入 isolate 往返或额外 `await`。
- **FR-006**: Rust 与 Dart 两侧的 `flutter_rust_bridge` 版本 MUST 严格一致（当前 `=2.13.0`）。
- **FR-007**: 全部绑定代码 MUST 由 `flutter_rust_bridge_codegen generate` 生成，生成物不得手改。
- **FR-008**: CI MUST 在 codegen 后校验 `apps/mobile/lib/src/rust` 与 `crates/mobile/src/frb_generated.rs`
  无差异，并执行 `flutter analyze` 与 `flutter test`。
- **FR-009**: CI MUST 构建 Android debug APK 与 iOS 无签名 debug 包；签名属于发布侧配置，不进 CI。

### Key Entities

- **绑定层 `crates/mobile`**: 仅转调 `app-core`，不含业务规则；产出 `cdylib` / `staticlib`。
- **Flutter 工程 `apps/mobile`**: 单屏 UI 与生成物；通过 `flutter_rust_bridge.yaml` 指向 workspace 根。

## Success Criteria

### Measurable Outcomes

- **SC-001**: 同一非法输入在 Dart、CLI、HTTP 三处的错误文案逐字相同，由各端测试共同守护。
- **SC-002**: `execute` 为同步调用，单次耗时不影响一帧预算（无 isolate、无额外拷贝）。
- **SC-003**: `flutter analyze` 零问题，widget 冒烟测试通过且不触发 FFI。
- **SC-004**: 忘记重新 codegen 时 CI 必然失败，绑定漂移不会进入主分支。

## Assumptions

- 单屏、无状态、无网络请求；不包含账号、离线缓存等后续能力。
- 发布签名、商店上架与 TestFlight 流程不在本期范围。
- 开发机需 Flutter SDK 与 `flutter_rust_bridge_codegen` 2.13.0，CI 已固定同一 Flutter 版本。
