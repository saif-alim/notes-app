use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use notes_api::{create_router, InMemoryNotesStore};
use tower::ServiceExt;

const MAX_BODY: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn app() -> axum::Router {
    let store = Arc::new(InMemoryNotesStore::new());
    create_router(store)
}

async fn post_note(app: axum::Router, body: &str) -> (StatusCode, serde_json::Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

// ---------------------------------------------------------------------------
// Original tests (kept intact)
// ---------------------------------------------------------------------------

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
    assert!(json["note"]["created_at_ms"].as_i64().unwrap() > 0);

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
    let (status, json) = post_note(app(), r#"{"body":""}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // Phase 5.5: errors are a JSON envelope, not a plain-text body.
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(json["error"]["message"], "body must not be empty");
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
        .map(|n| n["created_at_ms"].as_i64().unwrap())
        .collect();
    for w in times.windows(2) {
        assert!(w[0] <= w[1]);
    }
}

// ---------------------------------------------------------------------------
// New: Malformed / bad-request input
// ---------------------------------------------------------------------------

/// POST with malformed JSON must return 400 or 422, not 500.
#[tokio::test]
async fn post_malformed_json_is_rejected() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from("{not valid json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422, got {}",
        resp.status()
    );
}

/// POST with missing `body` field must return 400 or 422.
#[tokio::test]
async fn post_missing_body_field_is_rejected() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"title":"oops"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422, got {}",
        resp.status()
    );
}

/// POST with empty JSON object `{}` (missing field) must be rejected.
#[tokio::test]
async fn post_empty_json_object_is_rejected() {
    let (status, _) = post_note(app(), "{}").await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422, got {status}"
    );
}

/// POST with `body` set to JSON null must be rejected.
#[tokio::test]
async fn post_null_body_field_is_rejected() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"body":null}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422, got {}",
        resp.status()
    );
}

/// POST with `body` set to a number (type mismatch) must be rejected.
#[tokio::test]
async fn post_non_string_body_field_is_rejected() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"body":42}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "expected 400 or 422, got {}",
        resp.status()
    );
}

/// POST with no Content-Type header must not panic — expect 400 or 415.
#[tokio::test]
async fn post_no_content_type_is_rejected() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                // intentionally no content-type header
                .body(Body::from(r#"{"body":"hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "expected 400 or 415, got {}",
        resp.status()
    );
}

/// POST with wrong Content-Type must not panic — expect 400 or 415.
#[tokio::test]
async fn post_wrong_content_type_is_rejected() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "text/plain")
                .body(Body::from("body=hello"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "expected 400 or 415, got {}",
        resp.status()
    );
}

// ---------------------------------------------------------------------------
// New: Whitespace / trim semantics
// ---------------------------------------------------------------------------

/// Whitespace-only body (spaces) must be rejected (routes.rs trims before check).
/// Also validates the JSON error envelope shape introduced in Phase 5.5.
#[tokio::test]
async fn post_whitespace_only_body_is_rejected() {
    let (status, json) = post_note(app(), r#"{"body":"   "}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    assert_eq!(json["error"]["message"], "body must not be empty");
}

/// Whitespace-only body using tabs and newlines must also be rejected.
#[tokio::test]
async fn post_tab_newline_only_body_is_rejected() {
    let (status, _) = post_note(app(), "{\"body\":\"\\t\\n\"}").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// Surrounding whitespace is trimmed before storage. Fixed in Phase 5.5 after
/// qa-engineer review flagged the original behaviour (passed validation via
/// .trim() then stored raw) as asymmetric and client-hostile. `" hello "`
/// and `"hello"` now canonicalize to the same stored value.
#[tokio::test]
async fn post_body_with_surrounding_whitespace_is_trimmed() {
    let (status, json) = post_note(app(), r#"{"body":"  hello  "}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["note"]["body"], "hello");
}

// ---------------------------------------------------------------------------
// New: Unicode / non-ASCII content
// ---------------------------------------------------------------------------

/// Emoji body round-trips correctly.
#[tokio::test]
async fn post_emoji_body_round_trips() {
    let store = Arc::new(InMemoryNotesStore::new());
    let app = create_router(store);

    let (status, json) = post_note(app.clone(), r#"{"body":"🚀 launch"}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["note"]["body"], "🚀 launch");

    // Confirm it survives the list round-trip too.
    let resp = app
        .oneshot(Request::builder().uri("/notes").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let bytes = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list["notes"][0]["body"], "🚀 launch");
}

/// RTL text (Arabic) round-trips correctly.
#[tokio::test]
async fn post_rtl_unicode_body_round_trips() {
    let (status, json) = post_note(app(), r#"{"body":"مرحبا"}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["note"]["body"], "مرحبا");
}

/// CJK characters round-trip correctly.
#[tokio::test]
async fn post_cjk_body_round_trips() {
    let (status, json) = post_note(app(), r#"{"body":"日本語テスト"}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["note"]["body"], "日本語テスト");
}

// ---------------------------------------------------------------------------
// New: Large body
// ---------------------------------------------------------------------------

/// A very large body (just under serde's default limits) is accepted.
#[tokio::test]
async fn post_large_body_is_accepted() {
    let big: String = "a".repeat(10_000);
    let payload = format!(r#"{{"body":"{big}"}}"#);
    let (status, json) = post_note(app(), &payload).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["note"]["body"].as_str().unwrap().len(), 10_000);
}

// ---------------------------------------------------------------------------
// New: Unknown / extra fields
// ---------------------------------------------------------------------------

/// Extra unknown fields in the POST body are silently ignored (serde default).
/// This test documents the current permissive behaviour. If strict mode is
/// ever added (e.g. `#[serde(deny_unknown_fields)]`), this test should flip to 400.
#[tokio::test]
async fn post_extra_fields_are_ignored() {
    let (status, json) = post_note(app(), r#"{"body":"note","extra":"ignored","n":1}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["note"]["body"], "note");
    // extra fields must NOT appear in the response
    assert!(json["note"].get("extra").is_none());
    assert!(json["note"].get("n").is_none());
}

// ---------------------------------------------------------------------------
// New: Response shape invariants
// ---------------------------------------------------------------------------

/// POST 201 response includes all three required fields: id, body, created_at_ms.
#[tokio::test]
async fn post_response_shape_is_complete() {
    let (status, json) = post_note(app(), r#"{"body":"shape check"}"#).await;
    assert_eq!(status, StatusCode::CREATED);
    let note = &json["note"];
    // id must be a non-empty string (UUID format: 32 hex chars + 4 dashes = 36)
    let id = note["id"].as_str().expect("id must be a string");
    assert_eq!(id.len(), 36, "id should be a UUID (36 chars), got {id:?}");
    // Validate basic UUID structure: 8-4-4-4-12
    let parts: Vec<&str> = id.split('-').collect();
    assert_eq!(parts.len(), 5, "UUID must have 5 hyphen-separated groups");
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);

    // body echoed back exactly
    assert_eq!(note["body"], "shape check");

    // created_at_ms is a positive integer >= year-2020 epoch (milliseconds)
    let ts = note["created_at_ms"].as_i64().expect("created_at_ms must be i64");
    assert!(
        ts >= 1_577_836_800_000,
        "timestamp (ms) should be >= 2020-01-01: {ts}"
    );
}

/// GET /notes response has a `notes` array key at top level.
#[tokio::test]
async fn get_empty_store_response_shape() {
    let resp = app()
        .oneshot(Request::builder().uri("/notes").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // Must be an object with a `notes` array, not bare array or null.
    assert!(json.is_object(), "response must be a JSON object");
    assert!(json["notes"].is_array(), "`notes` must be an array");
    assert_eq!(json["notes"].as_array().unwrap().len(), 0);
}

/// Each note in the list response has the three required fields.
#[tokio::test]
async fn list_note_items_have_required_fields() {
    let store = Arc::new(InMemoryNotesStore::new());
    let app = create_router(store);

    let (status, _) = post_note(app.clone(), r#"{"body":"fields check"}"#).await;
    assert_eq!(status, StatusCode::CREATED);

    let resp = app
        .oneshot(Request::builder().uri("/notes").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let bytes = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let note = &json["notes"][0];
    assert!(note["id"].is_string(), "id must be a string");
    assert!(note["body"].is_string(), "body must be a string");
    assert!(note["created_at_ms"].is_number(), "created_at_ms must be a number");
}

// ---------------------------------------------------------------------------
// New: Concurrent writes
// ---------------------------------------------------------------------------

/// 20 concurrent POSTs to a shared store must all succeed and all appear in LIST.
/// Verifies RwLock-based store is safe under concurrent access.
#[tokio::test]
async fn concurrent_writes_all_persisted() {
    let store = Arc::new(InMemoryNotesStore::new());

    let mut set = tokio::task::JoinSet::new();
    for i in 0..20usize {
        let s = Arc::clone(&store);
        set.spawn(async move {
            let app = create_router(s);
            let payload = format!(r#"{{"body":"note-{i}"}}"#);
            let resp = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/notes")
                        .header("content-type", "application/json")
                        .body(Body::from(payload))
                        .unwrap(),
                )
                .await
                .unwrap();
            resp.status()
        });
    }

    let mut idx = 0usize;
    while let Some(res) = set.join_next().await {
        let status = res.expect("task panicked");
        assert_eq!(status, StatusCode::CREATED, "request {idx} did not get 201");
        idx += 1;
    }

    // All 20 notes must appear in the list.
    let list_app = create_router(store);
    let resp = list_app
        .oneshot(Request::builder().uri("/notes").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let bytes = to_bytes(resp.into_body(), MAX_BODY).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let count = json["notes"].as_array().unwrap().len();
    assert_eq!(count, 20, "expected 20 notes after concurrent writes, got {count}");
}

// ---------------------------------------------------------------------------
// New: Unknown routes
// ---------------------------------------------------------------------------

/// GET on an unknown path returns 404.
#[tokio::test]
async fn unknown_route_returns_404() {
    let resp = app()
        .oneshot(Request::builder().uri("/unknown").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// PUT /notes (method not allowed) returns 405.
#[tokio::test]
async fn put_notes_method_not_allowed() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/notes")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"body":"x"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

/// POST /notes with body >64KB (RequestBodyLimitLayer enforces MAX_BODY_BYTES) returns 413.
#[tokio::test]
async fn post_oversized_body_returns_413() {
    let huge_body: String = "a".repeat(65 * 1024); // 65KB, exceeds 64KB limit
    let payload = format!(r#"{{"body":"{}"}}"#, huge_body);
    let (status, _json) = post_note(app(), &payload).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}
