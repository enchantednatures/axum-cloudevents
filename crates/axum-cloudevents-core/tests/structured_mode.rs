use axum::{body::Body, extract::Request, http::{HeaderMap, header::StatusCode}, Request, response::Json};
use axum_cloudevents::{CloudEvent, CloudEventError, cloud_event};
use axum_test::{TestServerConfig, TestServer};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct TestData {
    message: String,
    count: i32,
}

#[tokio::test]
async fn test_structured_mode_with_data() {
    let json_event = json!({
        "specversion": "1.0",
        "type": "com.example.test",
        "source": "/test",
        "id": "test-123",
        "datacontenttype": "application/json",
        "data": {
            "message": "Hello",
            "count": 42
        }
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .json(&json_event)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_structured_mode_without_data() {
    let json_event = json!({
        "specversion": "1.0",
        "type": "com.example.heartbeat",
        "source": "/test",
        "id": "heartbeat-001"
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .json(&json_event)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_structured_mode_missing_required_field() {
    let json_event = json!({
        "specversion": "1.0",
        "source": "/test",
        "id": "test-123"
        // Missing "type" field
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .json(&json_event)
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_structured_mode_invalid_specversion() {
    let json_event = json!({
        "specversion": "0.3",
        "type": "com.example.test",
        "source": "/test",
        "id": "test-123"
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .json(&json_event)
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_structured_mode_with_extensions() {
    let json_event = json!({
        "specversion": "1.0",
        "type": "com.example.test",
        "source": "/test",
        "id": "test-456",
        "customext": "custom-value",
        "traceparent": "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
        "data": {
            "message": "test"
        }
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .json(&json_event)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cloud_event_response() {
    let response = TestServer::new(get_response_app())
        .await
        .get("/response")
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json();
    assert_eq!(body["type"], "com.example.response");
    assert_eq!(body["source"], "/test");
    assert!(body["id"].as_str().is_some());
    assert_eq!(body["data"]["message"], "Hello, World!");
}

async fn handle_event(event: CloudEvent<TestData>) -> Json<Value> {
    Json(json!({
        "status": "success",
        "message": event.data.message
    }))
}

async fn handle_event_with_opt(event: CloudEvent<Option<TestData>>) -> &'static str {
    match event.data {
        Some(data) => "has data",
        None => "no data",
    }
}

async fn handle_response() -> CloudEvent<TestData> {
    cloud_event(
        "com.example.response",
        "/test",
        TestData {
            message: "Hello, World!".to_string(),
            count: 1,
        }
    )
}

fn get_app() -> axum::Router {
    use axum::{routing::post, Router};
    Router::new()
        .route("/events", post(handle_event))
        .route("/opt", post(handle_event_with_opt))
}

fn get_response_app() -> axum::Router {
    use axum::{routing::get, Router};
    Router::new()
        .route("/response", get(handle_response))
}

#[derive(Deserialize)]
struct TestData {
    message: String,
    count: i32,
}
