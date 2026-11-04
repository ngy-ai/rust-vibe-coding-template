use std::net::SocketAddr;
use std::time::Instant;

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Extension, Request};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::post;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::dto::{ExecuteRequest, ExecuteResponse};
use crate::error::ApiError;

/// 请求链路 id 的响应头名，也是上游透传时使用的头名。
pub(crate) const REQUEST_ID_HEADER: &str = "x-request-id";

/// 单个请求的链路 id；由中间件写入请求扩展，handler 只读。
#[derive(Debug, Clone)]
pub(crate) struct RequestId(pub(crate) String);

/// 客户端传入的 id 允许的形态：可打印的 ASCII，长度有上限。
///
/// 上限与字符集是为了挡住把一个把日志行拆坏的值写进去（头注入，宪法 VIII）：
/// 换行、空格与超长串一律不认，直接换一个新的。
fn valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// 为每个请求确定链路 id，并把它写进响应头。
///
/// 优先沿用客户端给的合法 id，让上游的追踪串起来；缺失或非法时生成新的。
/// 响应头在错误分支同样存在，否则排障到一半就断了。
async fn with_request_id(mut request: Request, next: Next) -> Response {
    let incoming = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| valid_request_id(value))
        .map(str::to_owned);

    let id = incoming.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    request.extensions_mut().insert(RequestId(id.clone()));

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(REQUEST_ID_HEADER), value);
    }

    response
}

/// 构造 HTTP 路由。
///
/// 目前只有 `POST /api/execute` 一个端点。新增端点前先确认它能否并入这个端点
/// （宪法 IV），不要只为表面清晰而铺开一组路由。
pub fn router() -> Router {
    Router::new()
        .route("/api/execute", post(execute))
        .layer(middleware::from_fn(with_request_id))
}

/// 在 `addr` 上提供服务，直到收到 Ctrl+C。
///
/// 收到中断后停止接受新连接，让调用方在进程退出前 flush 文件日志。
///
/// # 错误
///
/// 地址不可用（端口被占用、没有绑定权限）时返回错误，信息包含地址与修正方向。
pub async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await.map_err(|error| {
        anyhow::anyhow!("无法监听 {addr}：{error}；请换一个端口，或用 --addr 指定其他地址")
    })?;
    let bound = listener.local_addr()?;

    info!(operation = "server.serve", address = %bound, "服务启动");

    axum::serve(listener, router())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!(operation = "server.serve", result = "ok", "服务已停止");
    Ok(())
}

/// 处理 `POST /api/execute`。
///
/// 请求体解析失败也走 [`ApiError`]，保证客户端拿到的错误结构与业务错误一致。
async fn execute(
    Extension(id): Extension<RequestId>,
    payload: Result<Json<ExecuteRequest>, JsonRejection>,
) -> Result<Json<ExecuteResponse>, ApiError> {
    let started = Instant::now();
    let Json(request) = payload.map_err(ApiError::from)?;
    let request_id = id.0.as_str();

    match app_core::execute(&request.input) {
        Ok(output) => {
            info!(
                operation = "http.execute",
                request_id,
                result = "ok",
                elapsed_us = started.elapsed().as_micros() as u64,
                "请求完成"
            );
            Ok(Json(ExecuteResponse {
                message: output.message,
            }))
        }
        Err(core_error) => {
            let api_error = ApiError::from(core_error);
            // 这里是错误的处理边界：只在被翻译成响应时记录一次，核心层不重复打点。
            warn!(
                operation = "http.execute",
                request_id,
                result = "rejected",
                kind = api_error.kind(),
                status = api_error.status().as_u16(),
                elapsed_us = started.elapsed().as_micros() as u64,
                "请求被拒绝"
            );
            Err(api_error)
        }
    }
}

/// 只在 Ctrl+C 时返回。
///
/// 信号注册失败时永久等待，宁可让进程被外部终止，也不要立刻退出而丢掉日志。
async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(error = %error, "监听中断信号失败，只能等待进程被外部终止");
        std::future::pending::<()>().await;
    }

    info!(operation = "server.shutdown", "收到中断信号，开始优雅退出");
}
