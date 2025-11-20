use crate::{
    graph,
    models::SystemState,
    oracle,
    risk::{risk_score, summarize_risk},
    risk_config::RiskConfig,
    scanner, utils,
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
    let path = PathBuf::from(".preflight/scan.json");

    let state_result = if !path.exists() {
        println!("No scan found — auto-running `preflight scan`...");

        let mut state = scanner::perform_scan();
        graph::derive_edges(&mut state);
        state.issues = oracle::evaluate(&state);
        state.assert_contract();

        if let Err(err) = utils::write_state(&path, &state) {
            eprintln!("Failed to persist auto-run scan: {err}");
        }

        Ok(state)
    } else {
        match fs::read_to_string(&path).await {
            Ok(contents) => serde_json::from_str::<SystemState>(&contents)
                .map_err(|err| format!("Corrupt scan.json: {err}")),
            Err(err) => Err(format!("Unable to read scan file: {err}")),
        }
    };

    match state_result {
        Ok(state) => {
            // Load dynamic risk config
            let cfg = RiskConfig::load();

            let total_risk = summarize_risk(&state.issues, &cfg);
            let issue_breakdown: Vec<(String, u32)> = state
                .issues
                .iter()
                .map(|issue| (issue.code.clone(), risk_score(issue, &cfg)))
                .collect();

            let mut val = serde_json::to_value(&state).expect("state serialize");

            val["risk_score_total"] = serde_json::json!(total_risk);
            val["risk_issue_breakdown"] = serde_json::json!(issue_breakdown);
            val["risk_config"] = serde_json::to_value(&cfg).unwrap();

            let etag_value = format!("W/\"{}-{}\"", state.timestamp, state.version);

            (StatusCode::OK, [(header::ETAG, etag_value)], Json(val)).into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
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
