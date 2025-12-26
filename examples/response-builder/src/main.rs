use axum::{Router, routing::{get, post}};
use axum_cloudevents::{CloudEvent, cloud_event, IntoCloudEventResponse};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserCreated {
    user_id: String,
    email: String,
    name: String,
}

// Using derive macro for automatic CloudEvent wrapping
#[derive(Serialize, IntoCloudEventResponse)]
#[cloud_event(type = "com.example.user.created", source = "/users")]
struct UserCreatedEvent {
    user_id: String,
    email: String,
    name: String,
}

// Manual CloudEvent construction using builder
async fn create_user() -> CloudEvent<UserCreated> {
    CloudEvent::builder()
        .id("user-123")
        .source("/users")
        .r#type("com.example.user.created")
        .time_now()
        .extension("tenant-id", "acme-corp")
        .data(UserCreated {
            user_id: "123".to_string(),
            email: "user@example.com".to_string(),
            name: "John Doe".to_string(),
        })
        .build()
        .unwrap()
}

// Using derive macro for simple responses
async fn get_user() -> UserCreatedEvent {
    UserCreatedEvent {
        user_id: "456".to_string(),
        email: "another@example.com".to_string(),
        name: "Jane Smith".to_string(),
    }
}

// Using helper function
async fn list_users() -> CloudEvent<Vec<UserCreated>> {
    cloud_event(
        "com.example.users.listed",
        "/users",
        vec![
            UserCreated {
                user_id: "001".to_string(),
                email: "user1@example.com".to_string(),
                name: "User One".to_string(),
            },
            UserCreated {
                user_id: "002".to_string(),
                email: "user2@example.com".to_string(),
                name: "User Two".to_string(),
            },
        ]
    )
}

// Echo handler that returns what it receives
async fn echo(event: CloudEvent<serde_json::Value>) -> CloudEvent<serde_json::Value> {
    event
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/users", get(list_users))
        .route("/echo", post(echo));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    tracing::info!("Server listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
