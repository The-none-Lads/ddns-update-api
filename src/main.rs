use axum::{
    routing::post,
    Router,
    http::{StatusCode, HeaderMap},
    extract::State,
};
use clap::Parser;
use tokio::process::Command;
use std::{net::SocketAddr, sync::Arc};

#[derive(Parser, Debug)]
#[command(name = "ddns-api", about = "Tiny DDNS update API")]
struct Args {
    /// Port to listen on
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Secret token required in x-auth-token header
    #[arg(long, default_value = "changeme")]
    secret: String,
}

#[derive(Clone)]
struct AppState {
    secret: String,
}

async fn update_ddns(headers: HeaderMap, State(state): State<Arc<AppState>>) -> (StatusCode, String) {
    let token = headers.get("x-auth-token").and_then(|v| v.to_str().ok());

    if token != Some(state.secret.as_str()) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized".into());
    }

    let output = Command::new("/usr/local/bin/update-ddns.sh")
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            (StatusCode::OK, String::from_utf8_lossy(&out.stdout).to_string())
        }
        Ok(out) => {
            (StatusCode::INTERNAL_SERVER_ERROR, String::from_utf8_lossy(&out.stderr).to_string())
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to run script: {}", e))
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let state = Arc::new(AppState {
        secret: args.secret.clone(),
    });

    let app = Router::new()
        .route("/update-ddns", post(update_ddns))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("🚀 DDNS API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
