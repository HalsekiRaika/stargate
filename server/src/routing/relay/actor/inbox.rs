use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use app_cmd::interactors::{DependOnRelayFollowAcceptInteractor, RelayFollowAcceptInteractor};
use kernel::entities::activity::types::Follow;
use kernel::entities::json::ActivityJson;
use crate::app::AppModule;

#[tracing::instrument(skip_all)]
pub async fn inbox(
    State(app): State<AppModule>,
    Json(json): Json<ActivityJson<Follow>>
) -> Result<StatusCode, StatusCode> {
    match app.relay_follow_accept_interactor()
        .execute(json)
        .await
    {
        Ok(_) => {}
        Err(reason) => {
            tracing::error!("Failed to process follow activity: {reason:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    Ok(StatusCode::ACCEPTED)
}