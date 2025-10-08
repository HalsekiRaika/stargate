use std::sync::LazyLock;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::Json;
use axum::response::IntoResponse;

use crate::app::AppModule;

#[tracing::instrument(skip_all)]
pub async fn profile(
    State(app): State<AppModule>
) -> impl IntoResponse {
    let actor = Json(serde_json::json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1",
        ],
        "type": "Service",
        "id": format!("https://{}/relay.actor", app.host_name()),
        "discoverable": true,
        "manuallyApprovesFollowers": false,
        "name": "relay.actor",
        "preferredUsername": "relay.actor",
        "following": format!("https://{}/relay.actor/following", app.host_name()),
        "followers": format!("https://{}/relay.actor/followers", app.host_name()),
        "inbox": format!("https://{}/relay.actor/inbox", app.host_name()),
        "outbox": format!("https://{}/relay.actor/outbox", app.host_name()),
        "publicKey": {
            "id": format!("https://{}/relay.actor#main-key", app.host_name()),
            "type": "Key",
            "owner": format!("https://{}/relay.actor", app.host_name()),
            "publicKeyPem": app.host_pubkey()
        },
    }));
    
    ([(CONTENT_TYPE, "application/activity+json")], actor)
}