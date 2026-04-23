mod fixtures;
mod http;

use axum::http::StatusCode;
use http::helpers::*;
use insta::assert_snapshot;
use lmah_inventory_rs::server::{database::insert::Insertable, routes::clients::client_router};
use sqlx;
use tower::ServiceExt;

use crate::fixtures::{clients::ClientFixture, make_state};

#[tokio::test]
async fn test_list_clients_empty() {
    let pool = create_test_db().await.unwrap();
    let app = client_router().with_state(make_state(pool).await);
    let request = get_request("/clients");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("list_clients_empty", body);
}

#[tokio::test]
async fn test_list_clients_multiple() {
    let pool = create_test_db().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    ClientFixture::alice().insert_one(&mut tx).await.unwrap();
    ClientFixture::bob().insert_one(&mut tx).await.unwrap();
    tx.commit().await.expect("Unable to commit the transaction");

    let app = client_router().with_state(make_state(pool).await);
    let request = get_request("/clients");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("Alice"));
    assert!(body.contains("Bob"));
    assert_snapshot!("list_clients_multiple", body);
}

#[tokio::test]
async fn test_list_clients_content() {
    let pool = create_test_db().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    ClientFixture::charlie().insert_one(&mut tx).await.unwrap();
    tx.commit().await.expect("Unable to commit the transaction");

    let app = client_router().with_state(make_state(pool).await);
    let request = get_request("/clients");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert!(body.contains("Charlie"));
    assert!(body.contains("Clark"));
    assert_snapshot!("list_clients_content", body);
}

#[tokio::test]
async fn test_new_client_form_renders() {
    let pool = create_test_db().await.unwrap();
    let app = client_router().with_state(make_state(pool).await);
    let request = get_request("/clients/new");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("new_client_form", body);
}

#[tokio::test]
async fn test_get_client_success() {
    let pool = create_test_db().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let client_id = ClientFixture::charlie()
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a client id after insert");
    tx.commit().await.expect("Unable to commit the transaction");

    let app = client_router().with_state(make_state(pool).await);
    let request = get_request(&format!("/clients/{}", client_id));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("get_client_success", body);
    assert!(body.contains("value=\"Charlie\""));
}

#[tokio::test]
async fn test_get_client_not_found() {
    let pool = create_test_db().await.unwrap();
    let app = client_router().with_state(make_state(pool).await);
    let request = get_request("/clients/999");

    let response = app.oneshot(request).await.unwrap();
    // TODO make a 404
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_create_client_success() {
    let pool = create_test_db().await.unwrap();
    let app = client_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("firstname", "Diana"),
        ("lastname", "Davis"),
        ("phone1", "(456) 789-0123"),
    ];
    let request = post_form_request("/clients/new", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = get_redirect_location(&response).unwrap();
    assert!(location.starts_with("/clients/"));
    assert!(location.contains("?success=true"));

    // Verify persisted
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE first_name = 'Diana'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_create_client_with_optional_fields() {
    let pool = create_test_db().await.unwrap();
    let app = client_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("firstname", "Frank"),
        ("lastname", "Foster"),
        ("street", "456 Oak Ave"),
        ("city", "Quebec"),
        ("phone1", "(678) 901-2345"),
        ("phone2", "(679) 901-2345"),
    ];
    let request = post_form_request("/clients/new", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // Verify optional fields saved
    let client: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT street, city FROM clients WHERE first_name = 'Frank'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(client.0, Some("456 Oak Ave".to_string()));
    assert_eq!(client.1, Some("Quebec".to_string()));
}

#[tokio::test]
async fn test_update_client_success() {
    let pool = create_test_db().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let client_id = ClientFixture::alice()
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a client id after insert");
    tx.commit().await.expect("Unable to commit the transaction");

    let app = client_router().with_state(make_state(pool.clone()).await);

    let form_data = [
        ("firstname", "Alicia"), // Changed
        ("lastname", "Anderson"),
        ("phone1", "(123) 456-7890"),
    ];
    let request = post_form_request(&format!("/clients/{}/update", client_id), &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let location = get_redirect_location(&response).unwrap();
    assert_eq!(location, format!("/clients/{}?success=true", client_id));

    // Verify update
    let name: String = sqlx::query_scalar("SELECT first_name FROM clients WHERE id = ?")
        .bind(client_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(name, "Alicia");
}

#[tokio::test]
async fn test_update_client_not_found() {
    let pool = create_test_db().await.unwrap();
    let app = client_router().with_state(make_state(pool).await);

    let form_data = [
        ("firstname", "Ghost"),
        ("lastname", "User"),
        ("phone1", "(999) 999-9999"),
    ];
    let request = post_form_request("/clients/999/update", &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_update_client_all_fields() {
    let pool = create_test_db().await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let client_id = ClientFixture::bob()
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a client id after insert");
    tx.commit().await.expect("Unable to commit the transaction");

    let app = client_router().with_state(make_state(pool.clone()).await);

    // Update all fields including optional ones
    let form_data = [
        ("firstname", "Robert"),
        ("lastname", "Brownson"),
        ("street", "789 Pine Rd"),
        ("city", "Montreal"),
        ("phone1", "(234) 567-8902"),
        ("phone2", "(235) 567-8902"),
    ];
    let request = post_form_request(&format!("/clients/{}/update", client_id), &form_data);

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    // Verify all fields updated
    let client: (String, String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT first_name, last_name, street, city FROM clients WHERE id = ?")
            .bind(client_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(client.0, "Robert");
    assert_eq!(client.1, "Brownson");
    assert_eq!(client.2, Some("789 Pine Rd".to_string()));
    assert_eq!(client.3, Some("Montreal".to_string()));
}
