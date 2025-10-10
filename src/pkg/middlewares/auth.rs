use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    tracing::debug!("req: {:?}", &request);
    next.run(request).await
}
