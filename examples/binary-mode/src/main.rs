use axum::{Router, routing::post, extract::State};
use axum_cloudevents::{CloudEvent, CloudEventConfig};
use serde::Deserialize;

#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    config: CloudEventConfig,
}

#[derive(Deserialize)]
struct BinaryEvent {
    message: String,
    #[allow(dead_code)]
    timestamp: String,
}

async fn handle_binary_event(
    State(_state): State<AppState>,
    event: CloudEvent<BinaryEvent>,
) -> &'static str {
    tracing::info!(
        event_id = %event.id(),
        event_type = %event.r#type(),
        source = %event.source(),
        "Received binary mode event: {}",
        event.data.message
    );

    "OK"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = CloudEventConfig {
        force_binary_mode: true,
        ..Default::default()
    };

    let app = Router::new()
        .route("/events", post(handle_binary_event))
        .with_state(AppState { config });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
