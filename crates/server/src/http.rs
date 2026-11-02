use std::net::SocketAddr;
use std::time::Instant;

use axum::Json;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::routing::post;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::dto::{ExecuteRequest, ExecuteResponse};
use crate::error::ApiError;

/// 构造 HTTP 路由。
///
/// 目前只有 `POST /api/execute` 一个端点。新增端点前先确认它能否并入这个端点
/// （宪法 IV），不要只为表面清晰而铺开一组路由。
pub fn router() -> Router {
    Router::new().route("/api/execute", post(execute))
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
    payload: Result<Json<ExecuteRequest>, JsonRejection>,
) -> Result<Json<ExecuteResponse>, ApiError> {
    let started = Instant::now();
    let Json(request) = payload.map_err(ApiError::from)?;

    match app_core::execute(&request.input) {
        Ok(output) => {
            info!(
                operation = "http.execute",
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
