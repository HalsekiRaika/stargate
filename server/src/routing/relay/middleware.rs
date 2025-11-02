use std::ops::Deref;
use axum::body::Body;
use axum::extract::{OriginalUri, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;
use driver::middleware::httpsig::{DependOnHttpSignatureVerifier, HttpSignatureVerifier};

use crate::app::AppModule;

pub async fn http_msgsign_verifier(
    State(app): State<AppModule>,
    mut req: Request,
    next: Next
) -> Result<Response, StatusCode> {
    if let Some(origin) = req.extensions()
        .get::<OriginalUri>()
        .map(|uri| uri.deref().to_owned())
    {
        *req.uri_mut() = origin
    }
    
    let req = match app
        .http_signature_verifier()
        .verify(req)
        .instrument(tracing::info_span!("middleware"))
        .await
    {
        Ok(req) => req.map(Body::new),
        Err(reason) => {
            tracing::warn!("Failed to verify HTTP signature: {reason:?}");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    Ok(next.run(req).await)
}