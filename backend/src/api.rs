//! HTTP API server for SaaS dashboard integration.
//!
//! Provides endpoints for health checks and WhatsApp QR pairing.
//! Spawned as a background task in the gateway, same pattern as scheduler/heartbeat.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use omega_channels::whatsapp::{self, WhatsAppChannel};
use omega_core::config::ApiConfig;
use omega_core::traits::Channel;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info};

/// Shared state for API handlers.
#[derive(Clone)]
pub struct ApiState {
    channels: HashMap<String, Arc<dyn Channel>>,
    api_key: Option<String>,
    uptime: Instant,
}

/// Check bearer token auth. Returns `None` if authorized, `Some(response)` if rejected.
fn check_auth(headers: &HeaderMap, api_key: &Option<String>) -> Option<(StatusCode, Json<Value>)> {
    let key = match api_key {
        Some(k) => k,
        None => return None, // No auth configured — allow all.
    };

    let header = match headers.get("authorization") {
        Some(h) => h,
        None => {
            return Some((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "missing Authorization header"})),
            ));
        }
    };

    let value = match header.to_str() {
        Ok(v) => v,
        Err(_) => {
            return Some((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "invalid Authorization header"})),
            ));
        }
    };

    match value.strip_prefix("Bearer ") {
        Some(token) if token == key => None, // Authorized.
        _ => Some((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid token"})),
        )),
    }
}

/// Downcast the WhatsApp channel from shared state.
fn get_whatsapp(state: &ApiState) -> Result<&WhatsAppChannel, (StatusCode, Json<Value>)> {
    let ch = state.channels.get("whatsapp").ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "WhatsApp channel not configured"})),
    ))?;

    ch.as_any().downcast_ref::<WhatsAppChannel>().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "WhatsApp channel downcast failed"})),
    ))
}

/// `GET /api/health` — Health check with uptime and WhatsApp status.
async fn health(
    headers: HeaderMap,
    State(state): State<ApiState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Some(err) = check_auth(&headers, &state.api_key) {
        return Err(err);
    }

    let uptime_secs = state.uptime.elapsed().as_secs();

    let whatsapp_status = match state.channels.get("whatsapp") {
        Some(ch) => match ch.as_any().downcast_ref::<WhatsAppChannel>() {
            Some(wa) => {
                if wa.is_connected().await {
                    "connected"
                } else {
                    "disconnected"
                }
            }
            None => "error",
        },
        None => "not_configured",
    };

    Ok(Json(json!({
        "status": "ok",
        "uptime_secs": uptime_secs,
        "whatsapp": whatsapp_status,
    })))
}

/// `POST /api/pair` — Trigger WhatsApp pairing, return QR as base64 PNG.
async fn pair(
    headers: HeaderMap,
    State(state): State<ApiState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Some(err) = check_auth(&headers, &state.api_key) {
        return Err(err);
    }

    let wa = get_whatsapp(&state)?;

    // Already paired — no need to generate QR.
    if wa.is_connected().await {
        return Ok(Json(json!({
            "status": "already_paired",
            "message": "WhatsApp is already connected",
        })));
    }

    // Restart bot for fresh QR codes.
    wa.restart_for_pairing().await.map_err(|e| {
        error!("WhatsApp restart_for_pairing failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("pairing restart failed: {e}")})),
        )
    })?;

    // Get receivers from the restarted bot.
    let (mut qr_rx, _done_rx) = wa.pairing_channels().await;

    // Wait up to 30s for the first QR code.
    let qr_data = tokio::time::timeout(std::time::Duration::from_secs(30), qr_rx.recv())
        .await
        .map_err(|_| {
            (
                StatusCode::GATEWAY_TIMEOUT,
                Json(json!({"error": "timed out waiting for QR code"})),
            )
        })?
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "QR channel closed unexpectedly"})),
        ))?;

    // Generate PNG and encode as base64.
    let png_bytes = whatsapp::generate_qr_image(&qr_data).map_err(|e| {
        error!("QR image generation failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("QR generation failed: {e}")})),
        )
    })?;

    let qr_base64 = BASE64.encode(&png_bytes);

    Ok(Json(json!({
        "status": "qr_ready",
        "qr_png_base64": qr_base64,
    })))
}

/// `GET /api/pair/status` — Long-poll (60s) for pairing completion.
async fn pair_status(
    headers: HeaderMap,
    State(state): State<ApiState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Some(err) = check_auth(&headers, &state.api_key) {
        return Err(err);
    }

    let wa = get_whatsapp(&state)?;

    // Already connected — immediate success.
    if wa.is_connected().await {
        return Ok(Json(json!({
            "status": "paired",
            "message": "WhatsApp is connected",
        })));
    }

    // Get done receiver and long-poll.
    let (_qr_rx, mut done_rx) = wa.pairing_channels().await;

    let paired = tokio::time::timeout(std::time::Duration::from_secs(60), done_rx.recv())
        .await
        .unwrap_or(Some(false))
        .unwrap_or(false);

    if paired {
        Ok(Json(json!({
            "status": "paired",
            "message": "WhatsApp pairing completed",
        })))
    } else {
        Ok(Json(json!({
            "status": "pending",
            "message": "Pairing not yet completed",
        })))
    }
}

/// Build the axum router with shared state.
fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/pair", post(pair))
        .route("/api/pair/status", get(pair_status))
        .with_state(state)
}

/// Start the API server. Called from `Gateway::run()`.
pub async fn serve(
    config: ApiConfig,
    channels: HashMap<String, Arc<dyn Channel>>,
    uptime: Instant,
) {
    let api_key = if config.api_key.is_empty() {
        None
    } else {
        Some(config.api_key.clone())
    };

    let state = ApiState {
        channels,
        api_key,
        uptime,
    };

    let app = build_router(state);
    let addr = format!("{}:{}", config.host, config.port);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("API server failed to bind to {addr}: {e}");
            return;
        }
    };

    info!("API server listening on {addr}");

    if let Err(e) = axum::serve(listener, app).await {
        error!("API server error: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// Build a test router with no channels (WhatsApp not configured).
    fn test_router(api_key: Option<String>) -> Router {
        let state = ApiState {
            channels: HashMap::new(),
            api_key,
            uptime: Instant::now(),
        };
        build_router(state)
    }

    #[tokio::test]
    async fn test_health_no_auth() {
        let app = test_router(None);
        let req = Request::get("/api/health").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["whatsapp"], "not_configured");
    }

    #[tokio::test]
    async fn test_health_valid_auth() {
        let app = test_router(Some("secret".to_string()));
        let req = Request::get("/api/health")
            .header("Authorization", "Bearer secret")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_bad_auth() {
        let app = test_router(Some("secret".to_string()));
        let req = Request::get("/api/health")
            .header("Authorization", "Bearer wrong")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_health_missing_auth() {
        let app = test_router(Some("secret".to_string()));
        let req = Request::get("/api/health").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_pair_no_whatsapp() {
        let app = test_router(None);
        let req = Request::post("/api/pair").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert!(json["error"].as_str().unwrap().contains("not configured"));
    }

    #[tokio::test]
    async fn test_pair_status_no_whatsapp() {
        let app = test_router(None);
        let req = Request::get("/api/pair/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
