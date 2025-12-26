use axum::{body::Body, extract::Request, http::{HeaderMap, header::{HeaderName, HeaderValue}, StatusCode}};
use axum_cloudevents::{CloudEvent, CloudEventConfig};
use axum_test::{TestServer, TestServerConfig};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
#[derive(Debug)]
struct BinaryTestData {
    message: String,
}

#[tokio::test]
async fn test_binary_mode_basic() {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("ce-specversion"), HeaderValue::from_static("1.0"));
    headers.insert(HeaderName::from_static("ce-type"), HeaderValue::from_static("com.example.test"));
    headers.insert(HeaderName::from_static("ce-source"), HeaderValue::from_static("/test"));
    headers.insert(HeaderName::from_static("ce-id"), HeaderValue::from_static("binary-123"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let body = json!({
        "message": "Binary mode test"
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .body(body.to_string())
        .headers(headers)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_binary_mode_with_optional_headers() {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("ce-specversion"), HeaderValue::from_static("1.0"));
    headers.insert(HeaderName::from_static("ce-type"), HeaderValue::from_static("com.example.test"));
    headers.insert(HeaderName::from_static("ce-source"), HeaderValue::from_static("/test"));
    headers.insert(HeaderName::from_static("ce-id"), HeaderValue::from_static("binary-456"));
    headers.insert(HeaderName::from_static("ce-subject"), HeaderValue::from_static("test-subject"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let body = json!({
        "message": "With optional headers"
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .body(body.to_string())
        .headers(headers)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_binary_mode_with_extensions() {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("ce-specversion"), HeaderValue::from_static("1.0"));
    headers.insert(HeaderName::from_static("ce-type"), HeaderValue::from_static("com.example.test"));
    headers.insert(HeaderName::from_static("ce-source"), HeaderValue::from_static("/test"));
    headers.insert(HeaderName::from_static("ce-id"), HeaderValue::from_static("binary-789"));
    headers.insert(
        HeaderName::from_static("ce-customext"),
        HeaderValue::from_static("custom-value")
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let body = json!({
        "message": "With extensions"
    });

    let response = Test_server::new(get_app())
        .await
        .post("/events")
        .body(body.to_string())
        .headers(headers)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_binary_mode_missing_required_header() {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("ce-specversion"), HeaderValue::from_static("1.0"));
    headers.insert(HeaderName::from_static("ce-source"), HeaderValue::from_static("/test"));
    headers.insert(HeaderName::from_static("ce-id"), HeaderValue::from_static("binary-123"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let body = json!({
        "message": "Test"
    });

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .body(body.to_string())
        .headers(headers)
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_binary_mode_empty_body() {
    let mut headers = HeaderMap::new();
    headers.insert(HeaderName::from_static("ce-specversion"), HeaderValue::from_static("1.0"));
    headers.insert(HeaderName::from_static("ce-type"), HeaderValue::from_static("com.example.heartbeat"));
    headers.insert(HeaderName::from_static("ce-source"), HeaderValue::from_static("/test"));
    headers.insert(HeaderName::from_static("ce-id"), HeaderValue::from_static("heartbeat-001"));
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    );

    let response = TestServer::new(get_app())
        .await
        .post("/events")
        .body("")
        .headers(headers)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}

async fn handle_binary_event(event: CloudEvent<BinaryTestData>) -> &'static str {
    "OK"
}

fn get_app() -> axum::Router {
    use axum::{routing::post, Router};
    Router::new().route("/events", post(handle_binary_event))
}
