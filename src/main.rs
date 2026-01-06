use axum::Json;
use axum::{Router, extract::State, http::StatusCode, routing::post};
use dotenvy;
use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;
use serde::Deserialize;
use std::sync::Arc;

struct AppState {
    resend: Resend,
}

#[derive(Deserialize)]
struct EmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init Resend SDK
    let resend_key = dotenvy::var("RESEND_API_KEY").expect("API key not present!");
    let resend_sdk = Resend::new(&resend_key);
    let shared_state = Arc::new(AppState { resend: resend_sdk });

    // init router, listener
    let app = Router::new()
        .route("/", post(endpoint))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Could not configure Tokio TCP listener!");
    // serve app
    axum::serve(listener, app)
        .await
        .expect("Could not start server! ⛔️");

    Ok(())
}

async fn endpoint(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailRequest>,
) -> Result<String, StatusCode> {
    let email = CreateEmailBaseOptions::new(&payload.from, payload.to, &payload.subject)
        .with_html("<strong>It works!</strong>");

    // access the state via the `State` extractor and handle the error
    match state.resend.emails.send(email).await {
        Ok(email) => Ok(email.id.to_string()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
