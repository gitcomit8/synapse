use axum::{
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use crate::AppState;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: axum::http::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement JWT validation
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify JWT token
    // TODO: Implement actual JWT verification

    Ok(next.run(request).await)
}
