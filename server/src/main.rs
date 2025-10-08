use axum::Router;
use axum::routing::{get, post};
use error_stack::{Report, ResultExt};
use tower::ServiceBuilder;
use server::{self, error::UnrecoverableError};

#[tokio::main]
async fn main() -> Result<(), Report<UnrecoverableError>> {
    let _guard = server::logging::setup();
    let config = driver::config::init_or_load("./config.toml")
        .change_context_lazy(|| UnrecoverableError)
        .attach("Failed config load.")?;
    
    let server_bind = (
        config.server.bind_address.clone(), 
        config.server.bind_port.unwrap_or(55555)
    );
    
    tracing::info!("Starting server at {}:{}", server_bind.0, server_bind.1);
    
    let app = server::app::init(config).await
        .attach("Failed initialization application module.")?;
    
    // Client Protocol
    let api = Router::new()
        .route("/api", get(|| async {  }));
    
    // ActivityPub Protocol
    let well_known = Router::new()
        .route("/webfinger", get(server::routing::relay::well_known::webfinger))
        .route("/nodeinfo", get(server::routing::relay::well_known::nodeinfo));
    
    let actor_proc = Router::new()
        .route("/inbox", post(server::routing::relay::actor::inbox))
        .route_layer(axum::middleware::from_fn_with_state(app.clone(), server::routing::relay::middleware::http_msgsign_verifier));
    
    let actor = Router::new()
        .route("/", get(server::routing::relay::actor::profile))
        .merge(actor_proc);
    
    let relay = Router::new()
        .nest("/.well-known", well_known)
        .nest("/relay.actor", actor);
    
    let root = Router::new()
        .merge(api)
        .merge(relay)
        .with_state(app);
    
    let tcpl = tokio::net::TcpListener::bind(&server_bind)
        .await
        .change_context_lazy(|| UnrecoverableError)
        .attach_with(|| format!("Unable bind {addr}:{port}", addr = server_bind.0, port = server_bind.1))?;
    
    axum::serve(tcpl, root)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .change_context_lazy(|| UnrecoverableError)?;
    
    Ok(())
}

async fn shutdown_signal() {
    let user_interrupt = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install keyboard interrupt.")
    };
    
    tokio::select! {
        _ = user_interrupt => {}
    }
}
