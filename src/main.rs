use axum::Json;
use axum::{Router, extract::State, http::StatusCode, routing::post};
use dotenvy;
use resend_rs::Resend;
use resend_rs::types::{CreateBroadcastOptions, SendBroadcastOptions};
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
        .route("/", post(post_endpoint))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1129")
        .await
        .expect("Could not configure Tokio TCP listener!");
    // serve app
    axum::serve(listener, app)
        .await
        .expect("Could not start server! ⛔️");

    Ok(())
}

async fn post_endpoint(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailRequest>,
) -> Result<(), StatusCode> {
    let resend_segment_id =
        dotenvy::var("RESEND_PROMPT_SEGMENT").expect("Prompt Segment ID not present!");
    let from = &payload.from;
    let subject = &payload.subject;

    let create_opts = CreateBroadcastOptions::new(&resend_segment_id, from, subject)
        .with_html("<strong>Yup.</strong>");

    let broadcast = state
        .resend
        .broadcasts
        .create(create_opts)
        .await
        .expect("Could not create Broadcast!");
    let send_opts = SendBroadcastOptions::new(&broadcast.id).with_scheduled_at("in 1 min");
    state
        .resend
        .broadcasts
        .send(send_opts)
        .await
        .expect("Could not send Broadcast!");
    Ok(())
}
