use app_core::CoreError;
use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// HTTP 层的错误：状态码加一套全项目统一的错误体。
///
/// 错误体固定为 `{"error": {"kind": "...", "message": "..."}}`，`kind` 供程序判断，
/// `message` 供人和 AI 读懂如何修正（宪法 IV）。
pub(crate) struct ApiError {
    status: StatusCode,
    kind: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    kind: &'a str,
    message: &'a str,
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    error: ErrorBody<'a>,
}

impl ApiError {
    pub(crate) fn status(&self) -> StatusCode {
        self.status
    }

    pub(crate) fn kind(&self) -> &'static str {
        self.kind
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        // 共享核心的校验失败一律是 422：请求语法合法，但内容不满足业务约束。
        let kind = match &error {
            CoreError::EmptyInput => "empty_input",
            CoreError::TooLong { .. } => "too_long",
        };

        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            kind,
            message: error.to_string(),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self {
            // axum 已经区分语法错误（400）与字段错误（422），沿用它的判定。
            status: rejection.status(),
            kind: "invalid_body",
            message: rejection.body_text(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: ErrorBody {
                kind: self.kind,
                message: &self.message,
            },
        };

        (self.status, Json(body)).into_response()
    }
}
