use axum::{Router, routing::{get, post}};
use axum_cloudevents::{CloudEvent, CloudEventTagged, cloud_event};
use serde::{Serialize, Deserialize};

// ============================================
// TIER 1: Direct Payload Extraction
// ============================================

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

#[derive(Deserialize)]
struct OrderPlaced {
    order_id: String,
    customer_id: String,
    items: Vec<OrderItem>,
    total: f64,
}

#[derive(Deserialize)]
struct OrderItem {
    product_id: String,
    quantity: u32,
    price: f64,
}

async fn handle_order(event: CloudEvent<OrderPlaced>) -> String {
    tracing::info!(
        event_id = %event.id(),
        order_id = %event.data.order_id,
        total = %event.data.total,
        "Order placed with {} items",
        event.data.items.len(),
    );

    format!("Order {} received", event.data.order_id)
}

// ============================================
// TIER 2: Tagged Enum Routing
// ============================================

#[derive(Deserialize, CloudEventTagged)]
#[cloud_event_prefix("com.example.payment")]
enum PaymentEvent {
    #[cloud_event_variant("processed")]
    Processed(PaymentProcessed),

    #[cloud_event_variant("failed")]
    Failed(PaymentFailed),

    #[cloud_event_variant("refunded")]
    Refunded(PaymentRefunded),

    #[cloud_event_catchall]
    Unknown(serde_json::Value),
}

#[derive(Deserialize)]
struct PaymentProcessed {
    payment_id: String,
    amount: f64,
    currency: String,
    transaction_id: String,
}

#[derive(Deserialize)]
struct PaymentFailed {
    payment_id: String,
    reason: String,
    error_code: String,
}

#[derive(Deserialize)]
struct PaymentRefunded {
    payment_id: String,
    refund_amount: f64,
    reason: String,
}

async fn handle_payment_events(event: CloudEvent<PaymentEvent>) -> &'static str {
    match &event.data {
        PaymentEvent::Processed(p) => {
            tracing::info!(
                payment_id = %p.payment_id,
                amount = %p.amount,
                currency = %p.currency,
                "Payment processed: {} {}",
                p.amount,
                p.currency
            );
        }
        PaymentEvent::Failed(f) => {
            tracing::warn!(
                payment_id = %f.payment_id,
                reason = %f.reason,
                error_code = %f.error_code,
                "Payment failed"
            );
        }
        PaymentEvent::Refunded(r) => {
            tracing::info!(
                payment_id = %r.payment_id,
                amount = %r.refund_amount,
                reason = %r.reason,
                "Payment refunded"
            );
        }
        PaymentEvent::Unknown(v) => {
            tracing::warn!(
                event_type = %event.r#type(),
                data = %v,
                "Unknown payment event type"
            );
        }
    }

    "OK"
}

// ============================================
// Response Builders
// ============================================

async fn create_event_response() -> CloudEvent<serde_json::Value> {
    cloud_event(
        "com.example.response",
        "/knative-template",
        serde_json::json!({
            "status": "success",
            "message": "Event processed successfully"
        })
    )
}

// ============================================
// Knative Health Endpoints
// ============================================

async fn health() -> &'static str {
    "OK"
}

async fn readiness() -> &'static str {
    "OK"
}

// ============================================
// Application Router
// ============================================

fn app() -> Router {
    Router::new()
        // Health endpoints (required for Knative)
        .route("/health", post(health))
        .route("/health/liveness", post(health))
        .route("/health/readiness", post(readiness))

        // Tier 1: Direct payload handlers
        .route("/users", post(handle_user_created))
        .route("/orders", post(handle_order))

        // Tier 2: Tagged enum handler
        .route("/payments", post(handle_payment_events))

        // Response example
        .route("/create-response", get(create_event_response))
}

#[tokio::main]
async fn main() {
    let app = app();

    // Knative uses $PORT environment variable
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Knative service on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
