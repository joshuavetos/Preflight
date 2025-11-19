use crate::models::SystemState;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use std::path::PathBuf;
use tokio::fs;
use tower_http::services::ServeDir;

async fn api_state_handler() -> impl IntoResponse {
    let path = PathBuf::from(".preflight/scan.json");
    if !path.exists() {
        return (
            StatusCode::NOT_FOUND,
            "Scan file not found. Run `preflight scan` first.",
        );
    }
    match fs::read_to_string(&path).await {
        Ok(contents) => match serde_json::from_str::<SystemState>(&contents) {
            Ok(state) => (StatusCode::OK, Json(state)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Corrupt scan.json: {err}"),
            )
                .into_response(),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unable to read scan file: {err}"),
        )
            .into_response(),
    }
}

fn dashboard_assets_root() -> Result<PathBuf, String> {
    let mut path = std::env::current_dir().map_err(|e| e.to_string())?;
    path.push("web/dist");
    let index = path.join("index.html");
    if !index.exists() {
        return Err(
            "Dashboard build not found. Run `npm install && npm run build` inside the /web directory.".to_string(),
        );
    }
    Ok(path)
}

pub async fn run_dashboard_server() -> Result<(), String> {
    let dist = dashboard_assets_root()?;
    let app = Router::new()
        .route("/api/state", get(api_state_handler))
        .fallback_service(ServeDir::new(dist).append_index_html_on_directories(true));

    let addr = "127.0.0.1:8787";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind dashboard port {addr}: {e}"))?;

    let url = format!("http://{addr}");
    println!("Dashboard available at {url}");

    let opener_url = url.clone();
    tokio::spawn(async move {
        if let Err(err) = open::that(opener_url) {
            eprintln!("Failed to open browser automatically: {err}");
        }
    });

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Dashboard server error: {e}"))
}
