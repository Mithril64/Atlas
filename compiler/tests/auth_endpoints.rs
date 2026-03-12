use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

#[path = "../src/auth.rs"]
mod auth;

#[tokio::test]
async fn profile_requires_authorization_header() {
    let app = auth::auth_router();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/profile")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    assert_eq!(text, "Unauthorized");
}
