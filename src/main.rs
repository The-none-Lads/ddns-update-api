use axum::{routing::post, Router, http::StatusCode};
use std::process::Command;
use std::net::SocketAddr;

async fn update_ddns() -> (StatusCode, String) {
    let output = Command::new("/usr/local/bin/update-ddns.sh")
        .output();

    match output {
        Ok(out) if out.status.success() => (StatusCode::OK, String::from_utf8_lossy(&out.stdout).to_string()),
        Ok(out) => (StatusCode::INTERNAL_SERVER_ERROR, String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to run script: {}", e)),
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid u16 number");
    let app = Router::new().route("/update-ddns", post(update_ddns));
    
    let addr = SocketAddr::from(([0,0,0,0], port));
    println!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}