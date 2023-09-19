use axum::{handler::HandlerWithoutStateExt, http::StatusCode, routing::get, Router};
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_PORT: u16 = 3004;

/// Serve static files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory of static files to serve
    #[arg(short, long)]
    dir: PathBuf,
    #[arg(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_static_file_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    println!("args: {:?}", args);
    let port = if let Some(port) = args.port {
        port
    } else {
        DEFAULT_PORT
    };
    let _ = tokio::spawn(serve(build_router(&args.dir), port)).await;
}
async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("listening on http://{}", addr);
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();
}
fn build_router(dir: &PathBuf) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }
    let serve_dir = ServeDir::new(dir).not_found_service(handle_404.into_service());
    Router::new().fallback_service(serve_dir)
}
