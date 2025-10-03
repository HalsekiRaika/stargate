use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use crate::app::AppModule;

pub async fn http_msgsign_verifier(
    State(app): State<AppModule>,
    mut req: Request, 
    next: Next
) -> Result<Response, StatusCode> {
    
    let res = next.run(req).await;
    Ok(res)
}