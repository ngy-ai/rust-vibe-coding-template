use serde::{Deserialize, Serialize};

/// `POST /api/execute` 的请求体。
///
/// `input` 的校验全部在 `app-core` 里完成，这一层不做第二遍判断，也不提供默认值：
/// 缺失字段应当由调用方修正，而不是被服务端悄悄补上。
#[derive(Debug, Deserialize)]
pub(crate) struct ExecuteRequest {
    /// 交给共享核心处理的文本；空白与超长都会被拒绝。
    pub(crate) input: String,
}

/// `POST /api/execute` 的成功响应。
#[derive(Debug, Serialize)]
pub(crate) struct ExecuteResponse {
    /// 与 CLI 的 stdout 完全相同的一行结果，便于各端断言同一份契约。
    pub(crate) message: String,
}
