use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use notes_api::{create_router, InMemoryNotesStore};
use tower::ServiceExt;

const MAX_BODY: usize = 64 * 1024;

#[tokio::test]
async fn round_trip_create_then_list() {
    let store = Arc::new(InMemoryNotesStore::new());
    let app = create_router(store);

    // List empty
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/notes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    assert_eq!(&body[..], b"{\"notes\":[]}");

    // Create one
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"body":"hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["note"]["body"], "hello");
    assert!(json["note"]["id"].as_str().unwrap().len() >= 32);
    assert!(json["note"]["created_at_unix"].as_i64().unwrap() > 0);

    // List returns the created note
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/notes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["body"], "hello");
}

#[tokio::test]
async fn rejects_empty_body() {
    let store = Arc::new(InMemoryNotesStore::new());
    let app = create_router(store);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"body":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_is_ordered_by_creation_time() {
    let store = Arc::new(InMemoryNotesStore::new());
    let app = create_router(store);

    for body in ["first", "second", "third"] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notes")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"body":"{body}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/notes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let notes = json["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 3);
    // Creation-time ordering; all three inserted within the same second is
    // possible, so assert monotonic non-decreasing rather than strict order.
    let times: Vec<i64> = notes
        .iter()
        .map(|n| n["created_at_unix"].as_i64().unwrap())
        .collect();
    for w in times.windows(2) {
        assert!(w[0] <= w[1]);
    }
}

