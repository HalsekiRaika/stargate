use axum::Router;
use axum::routing::get;
use error_stack::{Report, ResultExt};
use server::{self, error::UnrecoverableError};

#[tokio::main]
async fn main() -> Result<(), Report<UnrecoverableError>> {
    let _guard = server::logging::setup();
    let config = driver::config::init_or_load("../config.toml")
        .change_context_lazy(|| UnrecoverableError)
        .attach("Failed config load.")?;
    
    let server_bind = (
        config.server.bind_address.clone(), 
        config.server.bind_port.unwrap_or(55555)
    );
    
    let app = server::app::init(config).await
        .attach("Failed initialization application module.")?;
    
    // ActivityPub Protocol
    let relay = Router::new()
        .route("/.well-known", get(|| async {  }))
        .route("/relay.actor", get(|| async {  }));
    
    // Client Protocol
    let api = Router::new()
        .route("/api", get(|| async {  }));
    
    let root = Router::new()
        .merge(relay)
        .merge(api)
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
