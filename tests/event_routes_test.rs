mod fixtures;
mod http;

use axum::http::StatusCode;
use fixtures::events::EventFixture;
use http::helpers::*;
use insta::assert_snapshot;
use lmah_inventory_rs::server::models::events::EventForm;
use lmah_inventory_rs::server::routes::events::event_router;
use lmah_inventory_rs::server::services::events::insert_event;
use tower::ServiceExt;

use crate::fixtures::make_state;

// GET /events tests

#[tokio::test]
async fn test_list_events_empty() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool).await);
    let request = get_request("/events");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("list_events_empty", body);
}

#[tokio::test]
async fn test_list_events_multiple() {
    let pool = create_test_db().await.unwrap();
    let wedding_form = EventForm {
        name: EventFixture::wedding().name,
        date: EventFixture::wedding().date,
        event_type: EventFixture::wedding().event_type,
    };
    let prom_form = EventForm {
        name: EventFixture::prom().name,
        date: EventFixture::prom().date,
        event_type: EventFixture::prom().event_type,
    };
    insert_event(&pool, wedding_form).await.unwrap();
    insert_event(&pool, prom_form).await.unwrap();

    let app = event_router().with_state(make_state(pool).await);
    let request = get_request("/events");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("Wedding"));
    assert!(body.contains("Prom"));
    assert_snapshot!("list_events_multiple", body);
}

#[tokio::test]
async fn test_list_events_content() {
    let pool = create_test_db().await.unwrap();
    let gala_form = EventForm {
        name: EventFixture::gala().name,
        date: EventFixture::gala().date,
        event_type: EventFixture::gala().event_type,
    };
    insert_event(&pool, gala_form).await.unwrap();

    let app = event_router().with_state(make_state(pool).await);
    let request = get_request("/events");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("Charity Gala"));
    assert_snapshot!("list_events_content", body);
}

// GET /events/new tests

#[tokio::test]
async fn test_new_event_form_renders() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool).await);
    let request = get_request("/events/new");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("new_event_form", body);
}

#[tokio::test]
async fn test_new_event_form_fields() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool).await);
    let request = get_request("/events/new");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert!(body.contains(r#"name="name""#));
    assert!(body.contains(r#"name="date""#));
    assert!(body.contains(r#"name="type""#));
}

// GET /events/{id} tests

#[tokio::test]
async fn test_get_event_success() {
    let pool = create_test_db().await.unwrap();
    let gala_form = EventForm {
        name: EventFixture::gala().name,
        date: EventFixture::gala().date,
        event_type: EventFixture::gala().event_type,
    };
    let event_id = insert_event(&pool, gala_form).await.unwrap();

    let app = event_router().with_state(make_state(pool).await);
    let request = get_request(&format!("/events/{}", event_id));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("get_event_success", body);
    assert!(body.contains("value=\"Charity Gala\""));
}

#[tokio::test]
async fn test_get_event_not_found() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool).await);
    let request = get_request("/events/999");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// POST /events/new tests

#[tokio::test]
async fn test_create_event_success() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("name", "Birthday Party"),
        ("date", "2026-07-01"),
        ("type", "Birthday"),
    ];
    let request = post_form_request("/events/new", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = get_redirect_location(&response).unwrap();
    assert!(location.starts_with("/events/"));
    assert!(location.contains("?success=true"));
}

#[tokio::test]
async fn test_create_event_persists() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("name", "Birthday Party"),
        ("date", "2026-07-01"),
        ("type", "Birthday"),
    ];
    let request = post_form_request("/events/new", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // Verify persisted
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE name = 'Birthday Party'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_create_event_all_fields() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("name", "Corporate Gala"),
        ("date", "2026-08-15"),
        ("type", "Corporate"),
    ];
    let request = post_form_request("/events/new", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // Verify all fields saved
    let event: (String, String, String) =
        sqlx::query_as("SELECT name, event_type, date FROM events WHERE name = 'Corporate Gala'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(event.0, "Corporate Gala");
    assert_eq!(event.1, "Corporate");
    assert_eq!(event.2, "2026-08-15");
}

#[tokio::test]
async fn test_create_event_special_characters() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("name", "Spring Festival & Concert"),
        ("date", "2026-09-01"),
        ("type", "Festival"),
    ];
    let request = post_form_request("/events/new", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // Verify persists correctly
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE name = 'Spring Festival & Concert'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

// POST /events/{id}/update tests

#[tokio::test]
async fn test_update_event_success() {
    let pool = create_test_db().await.unwrap();
    let wedding_form = EventForm {
        name: EventFixture::wedding().name,
        date: EventFixture::wedding().date,
        event_type: EventFixture::wedding().event_type,
    };
    let event_id = insert_event(&pool, wedding_form).await.unwrap();

    let app = event_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("name", "Smith-Garcia Wedding"), // Changed
        ("date", "2026-06-15"),
        ("type", "Wedding"),
    ];
    let request = post_form_request(&format!("/events/{}/update", event_id), &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = get_redirect_location(&response).unwrap();
    assert_eq!(location, format!("/events/{}?success=true", event_id));

    // Verify update
    let name: String = sqlx::query_scalar("SELECT name FROM events WHERE id = ?")
        .bind(event_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(name, "Smith-Garcia Wedding");
}

#[tokio::test]
async fn test_update_event_not_found() {
    let pool = create_test_db().await.unwrap();
    let app = event_router().with_state(make_state(pool).await);

    let form_data = [
        ("name", "Ghost Event"),
        ("date", "2026-12-31"),
        ("type", "Mystery"),
    ];
    let request = post_form_request("/events/999/update", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_update_event_all_fields() {
    let pool = create_test_db().await.unwrap();
    let prom_form = EventForm {
        name: EventFixture::prom().name,
        date: EventFixture::prom().date,
        event_type: EventFixture::prom().event_type,
    };
    let event_id = insert_event(&pool, prom_form).await.unwrap();

    let app = event_router().with_state(make_state(pool.clone()).await);

    // Update all fields
    let form_data = [
        ("name", "Fall Prom 2026"),
        ("date", "2026-10-20"),
        ("type", "Prom"),
    ];
    let request = post_form_request(&format!("/events/{}/update", event_id), &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // Verify all fields updated
    let event: (String, String, String) =
        sqlx::query_as("SELECT name, event_type, date FROM events WHERE id = ?")
            .bind(event_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(event.0, "Fall Prom 2026");
    assert_eq!(event.1, "Prom");
    assert_eq!(event.2, "2026-10-20");
}
