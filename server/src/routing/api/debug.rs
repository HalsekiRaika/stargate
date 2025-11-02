use axum::extract::Request;

pub async fn debug(request: Request) {
    tracing::debug!("{request:#?}");
}