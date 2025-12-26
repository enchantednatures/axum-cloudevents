use axum::{Router, routing::post};
use axum_cloudevents::{CloudEvent, CloudEventTagged};
use serde::Deserialize;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/payments", post(handle_payment_events));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
