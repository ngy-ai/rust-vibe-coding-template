# Feature Specification: 共享核心的 HTTP 服务端

**Feature Branch**: `refactor/workspace-split`

**Created**: 2026-11-02

**Status**: 已实现（`spec.md` 于 2026-11-03 回溯补写，描述上线行为；实现计划见 [plan.md](./plan.md)）

**Input**: 让 PC 端与移动端之外的调用方通过 HTTP 复用 `app-core` 的输入校验与文案

## User Scenarios & Testing

### User Story 1 - 远程调用方复用共享校验 (Priority: P1)

调用方把一段文本 `POST` 到 `/api/execute`，服务端用与 CLI 完全相同的规则校验并返回一行结果；
输入不合法时返回结构化错误体，其中的 `message` 与 CLI 的错误输出逐字一致。

**Why this priority**: 这是本服务的唯一用途。没有它，服务端只是一层无意义的转发壳。

**Independent Test**: 用 `tower::ServiceExt::oneshot` 直接调用路由（不绑定端口），断言 200 的
`message` 与 422 的 `error.kind` / `error.message`。

**Acceptance Scenarios**:

1. **Given** 输入 `"hello"`，**When** `POST /api/execute`，**Then** 返回 200 与 `{"message":"已处理 5 个字符"}`
2. **Given** 输入只有空白字符，**When** 提交，**Then** 返回 422，`error.kind = "empty_input"`，
   `error.message` 说明可执行的修正动作
3. **Given** 输入 4097 个字符，**When** 提交，**Then** 返回 422，`error.kind = "too_long"`，
   `message` 同时给出实际字符数与上限
4. **Given** 请求体缺少 `input` 字段或类型不符，**When** 提交，**Then** 返回 422，
   `error.kind = "invalid_body"`
5. **Given** 请求体不是合法 JSON，**When** 提交，**Then** 返回 400，`error.kind = "invalid_body"`

---

### User Story 2 - 运维者安全地起停服务 (Priority: P2)

部署者用一条命令启动服务，默认只监听回环地址；需要对外时显式指定地址。收到中断信号后优雅退出，
不在控制台泄漏日志噪声。

**Why this priority**: 默认监听 `0.0.0.0` 会让未鉴权的端点在局域网裸奔；退出时丢日志会让排障断档。

**Independent Test**: 以默认参数启动，确认监听 `127.0.0.1:8080`；占用端口后再启动，确认错误信息
包含地址与修正方向。

**Acceptance Scenarios**:

1. **Given** 不传参数，**When** 启动服务，**Then** 监听 `127.0.0.1:8080`
2. **Given** 端口已被占用，**When** 启动，**Then** 进程以非零退出，错误信息包含该地址与「换端口或用 `--addr`」
3. **Given** 服务运行中，**When** 收到 Ctrl+C，**Then** 停止接受新连接并完成文件日志 flush 后退出

---

### Edge Cases

- 输入首尾有空白：先 `trim` 再计数，`"  hi  "` 记为 2 个字符。
- 多字节字符：上限按 Unicode 标量计数，4096 个汉字不算超限。
- 超长输入的错误信息必须带具体数字，不能只说「超长」。
- 请求被拒绝时日志只记录一次（发生在错误被翻译成响应的边界）。
- 客户端传入的 `x-request-id` 含空格、换行或超过 64 字符：不识别为 id，直接生成新的，
  避免把一个能拆坏日志行的值写进结构化日志。

## Requirements

### Functional Requirements

- **FR-001**: 服务 MUST 提供 `POST /api/execute`，请求体为 `{"input": string}`。
- **FR-002**: 成功时 MUST 返回 200 与 `{"message": string}`，其 `message` 与 CLI 写入 stdout 的内容逐字相同。
- **FR-003**: 空白输入 MUST 返回 422，`error.kind = "empty_input"`。
- **FR-004**: 超过 4096 个 Unicode 标量的输入 MUST 返回 422，`error.kind = "too_long"`，消息含实际字符数与上限。
- **FR-005**: 请求体解析失败 MUST 返回统一的错误体，`error.kind = "invalid_body"`；状态码沿用解析层
  已有的判定（语法错误 400、字段错误 422）。
- **FR-006**: 所有错误响应 MUST 使用固定结构 `{"error": {"kind": string, "message": string}}`，
  `message` 必须指出输入哪里不合法以及如何修正。
- **FR-007**: 业务校验 MUST 只在 `app-core` 中出现一次；服务端不得做第二遍校验，也不得为缺失字段补默认值。
- **FR-008**: 服务 MUST 默认监听 `127.0.0.1:8080`，对外暴露必须由 `--addr` 显式指定。
- **FR-009**: 服务 MUST 在收到中断信号后优雅退出，并在退出前完成日志 flush。
- **FR-010**: stdout MUST 只承载业务数据，日志一律不写入 stdout。
- **FR-011**: 每个请求 MUST 记录 `operation`、`result`、`elapsed_us`；被拒绝的请求额外记录 `kind` 与 `status`，
  且只记录一次。
- **FR-012**: 每个响应 MUST 带 `x-request-id` 头：客户端提供了合法值就原样回显，缺失或非法则生成新值；
  合法值限定为不超过 64 个字符的 ASCII 字母、数字与 `-` `_` `.`。日志记录同一个 id，
  成功与拒绝分支都要有，错误响应也不例外。

### Key Entities

- **ExecuteRequest**: `input`（必填，无默认值）。
- **ExecuteResponse**: `message`，与 CLI 输出同源。
- **ErrorBody**: `kind`（程序判断用，稳定字符串）+ `message`（人和 AI 读懂修正动作）。

## Success Criteria

### Measurable Outcomes

- **SC-001**: 同一输入经 CLI 与经 HTTP 得到的成功文案、错误文案逐字相同，由契约测试断言。
- **SC-002**: 路由契约测试覆盖成功、两类业务拒绝、缺字段、语法错误共 5 条，全部通过且不绑定端口。
- **SC-003**: `app-core::execute` 的 p95 < 1ms（宪法 V 的库默认预算）。
- **SC-004**: CI 的格式、lint、测试、治理扫描、机密扫描与依赖审计六类门禁全部通过。
- **SC-005**: 每条请求在响应头与日志里携带同一个 `x-request-id`，错误响应同样如此；
  由 5 条契约测试守护（生成、回显、非法替换、超长替换、错误响应保留）。

## Assumptions

- 模板演示用途，端点不做认证与限流；对外暴露由部署方自行加一层网关。
- 无持久化、无状态，不引入数据库或缓存。
- 只有一个端点：新增端点前先确认能否并入 `/api/execute`（宪法 IV）。
