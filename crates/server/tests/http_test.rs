use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// 对路由直接发一次 POST，返回状态码与解析后的 JSON。
///
/// 不绑定套接字：端口分配与防火墙会让 CI 上的端口测试变成随机失败源。
async fn post_execute(body: &str) -> (StatusCode, serde_json::Value) {
    let response = app_server::router()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/execute")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("请求应当可以构造"),
        )
        .await
        .expect("路由不应当返回错误");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("响应体应当可读")
        .to_bytes();
    let value = serde_json::from_slice(&bytes).expect("响应应当是 JSON");

    (status, value)
}

#[tokio::test]
async fn returns_message_for_valid_input() {
    let (status, body) = post_execute(r#"{"input":"hello"}"#).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["message"], "已处理 5 个字符");
}

#[tokio::test]
async fn rejects_blank_input() {
    let (status, body) = post_execute(r#"{"input":"   "}"#).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["kind"], "empty_input");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message 应当是字符串")
            .contains("输入不能为空")
    );
}

#[tokio::test]
async fn rejects_input_over_limit() {
    let body = serde_json::json!({ "input": "a".repeat(app_core::MAX_INPUT_CHARS + 1) });

    let (status, body) = post_execute(&body.to_string()).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["kind"], "too_long");
}

#[tokio::test]
async fn rejects_missing_field_with_same_error_shape() {
    let (status, body) = post_execute(r#"{"inpt":"hello"}"#).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["kind"], "invalid_body");
}

#[tokio::test]
async fn rejects_malformed_json_with_same_error_shape() {
    let (status, body) = post_execute(r#"{"input":"#).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["kind"], "invalid_body");
}
