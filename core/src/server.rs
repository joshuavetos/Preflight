use crate::{
    config::RiskConfig,
    models::SystemState,
    risk::{risk_score, summarize_risk},
};
use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::path::PathBuf;
use tokio::fs;
use tower_http::services::ServeDir;

async fn api_state_handler() -> impl IntoResponse {
    let cfg = match RiskConfig::load() {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let path = PathBuf::from(".preflight/scan.json");
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Scan file not found. Run `preflight scan` first.")
            .into_response();
    }

    match fs::read_to_string(&path).await {
        Ok(contents) => match serde_json::from_str::<SystemState>(&contents) {
            Ok(state) => {
                let total_risk = summarize_risk(&state.issues, &cfg);
                let breakdown: Vec<(String, u32)> = state
                    .issues
                    .iter()
                    .map(|i| (i.code.clone(), risk_score(i, &cfg)))
                    .collect();

                let mut val = serde_json::to_value(&state).unwrap();
                val["risk_score_total"] = serde_json::json!(total_risk);
                val["risk_issue_breakdown"] = serde_json::json!(breakdown);

                let etag_value = format!("W/\"{}-{}\"", state.timestamp, state.version);
                (StatusCode::OK, [(header::ETAG, etag_value)], Json(val)).into_response()
            }
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
    if !path.join("index.html").exists() {
        return Err("Dashboard build missing — run `npm run build` inside /web".to_string());
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

    println!("Dashboard available at http://{addr}");
    println!("Press Ctrl+C to stop the dashboard server.");

    tokio::spawn(async move {
        if let Err(err) = open::that(format!("http://{addr}")) {
            eprintln!("Failed to open browser automatically: {err}");
        }
    });

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Dashboard server error: {e}"))
}
