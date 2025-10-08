use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use driver::middleware::httpsig::{DependOnHttpSignatureVerifier, HttpSignatureVerifier};

use crate::app::AppModule;

#[tracing::instrument(skip_all)]
pub async fn http_msgsign_verifier(
    State(app): State<AppModule>,
    req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    let req = match app
        .http_signature_verifier()
        .verify(req)
        .await
    {
        Ok(req) => {
            req.map(Body::new)
        }
        Err(reason) => {
            tracing::warn!("Failed to verify HTTP signature: {reason:?}");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    let res = next.run(req).await;
    
    Ok(res)
}