use axum::extract::State;
use axum::Json;

use crate::app::AppModule;

pub async fn webfinger(
    State(app): State<AppModule>
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "subject": format!("acct:relay.actor@{}", app.host_name()),
        "aliases": [
            format!("https://{}/relay.actor", app.host_name())
        ],
        "links": [
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": format!("https://{}/relay.actor", app.host_name())
            }
        ]
    }))
}

pub async fn nodeinfo() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "openRegistration": false,
        "version": "2.1",
        "protocols": [
            "activitypub"
        ],
        "software": {
            "name": "stargate",
            "version": "0.1.0"
        },
        "usage": {
            "users": {
                "total": 1
            }
        },
    }))
}
