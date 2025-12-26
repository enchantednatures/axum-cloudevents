use axum::{Router, routing::post};
use axum_cloudevents::CloudEvent;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserCreated {
    user_id: String,
    email: String,
    name: String,
}

async fn handle_user_created(event: CloudEvent<UserCreated>) -> &'static str {
    tracing::info!(
        event_id = %event.id(),
        event_type = %event.r#type(),
        source = %event.source(),
        "User {} created: {} ({})",
        event.data.user_id,
        event.data.name,
        event.data.email,
    );

    "OK"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/events", post(handle_user_created));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
