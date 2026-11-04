//! 请求链路 id 的契约测试（宪法 VII）。
//!
//! 规则：每个响应都带 `x-request-id`；客户端给了合法 id 就沿用，非法或缺失就换一个新的。
//! 这样一条请求在日志、响应头与上游系统里能用同一个字符串对上。

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use tower::ServiceExt;

/// 发一次 `POST /api/execute`，返回状态码与响应头。
async fn post_with(headers: &[(&str, &str)], body: &str) -> (StatusCode, HeaderMap) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }

    let response = app_server::router()
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("请求应当可以构造"),
        )
        .await
        .expect("路由不应当返回错误");

    (response.status(), response.headers().clone())
}

fn request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let (status, headers) = post_with(&[], r#"{"input":"hello"}"#).await;

    assert_eq!(status, StatusCode::OK);
    let id = request_id(&headers).expect("响应必须带 x-request-id");
    assert!(!id.is_empty(), "请求 id 不得为空");
    assert!(
        id.len() >= 16,
        "自动生成的 id 应当足够长以免碰撞，实际: {id}"
    );
}

#[tokio::test]
async fn echoes_client_supplied_id() {
    let (status, headers) =
        post_with(&[("x-request-id", "trace-abc-123")], r#"{"input":"hello"}"#).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(request_id(&headers).as_deref(), Some("trace-abc-123"));
}

#[tokio::test]
async fn replaces_invalid_client_id() {
    // 含空格会把日志行拆坏，也是常见的头注入尝试。
    let (status, headers) = post_with(
        &[("x-request-id", "bad id with spaces")],
        r#"{"input":"hello"}"#,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let id = request_id(&headers).expect("响应必须带 x-request-id");
    assert_ne!(id, "bad id with spaces", "非法 id 不得被回显");
}

#[tokio::test]
async fn replaces_over_long_client_id() {
    let long = "a".repeat(200);
    let (status, headers) = post_with(&[("x-request-id", &long)], r#"{"input":"hello"}"#).await;

    assert_eq!(status, StatusCode::OK);
    let id = request_id(&headers).expect("响应必须带 x-request-id");
    assert_ne!(id, long, "超长 id 不得被回显");
    assert!(id.len() <= 64, "回显的 id 必须有长度上限，实际: {id}");
}

#[tokio::test]
async fn keeps_request_id_on_error_responses() {
    let (status, headers) = post_with(&[], r#"{"input":"   "}"#).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        request_id(&headers).is_some(),
        "错误响应同样要带请求 id，否则排障时断链"
    );
}
