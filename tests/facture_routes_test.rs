mod fixtures;
mod http;

use axum::http::StatusCode;
use http::helpers::*;
use insta::assert_snapshot;
use lmah_inventory_rs::server::{database::insert::Insertable, routes::factures::facture_router};
use tower::ServiceExt;

use crate::fixtures::{
    clients::ClientFixture,
    factures::{
        FactureFixture, FactureItemFixture, ProductFixture, ProductTypeFixture, StatutFixture,
    },
    make_state,
};

#[tokio::test]
async fn test_list_factures_empty() {
    let pool = create_test_db().await.unwrap();
    let app = facture_router().with_state(make_state(pool).await);
    let request = get_request("/factures");

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = body_to_string(response.into_body()).await;
    assert_snapshot!("list_factures_empty", body);
}

#[tokio::test]
async fn test_list_factures_with_data() {
    let pool = create_test_db().await.unwrap();
    let mut tx = pool.begin().await.unwrap();

    // Insert client
    let client_id = ClientFixture::alice()
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a client id after insert");

    // Insert product type
    ProductTypeFixture::wedding_dress()
        .insert_one(&mut tx)
        .await
        .unwrap();

    // Insert product
    let product_id = ProductFixture::evening_dress()
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a product id after insert");

    // Link product to product type
    sqlx::query("INSERT INTO product_product_types (product_id, product_type_name) VALUES (?, ?)")
        .bind(product_id)
        .bind("Robe de mariée")
        .execute(&mut *tx)
        .await
        .expect("Unable to link product to product type");

    // Insert facture
    let facture_id = FactureFixture::simple_product_facture(client_id, product_id)
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a facture id after insert");

    // Insert facture item
    let facture_item_id = FactureItemFixture::product_item(facture_id, product_id)
        .insert_one(&mut tx)
        .await
        .unwrap()
        .expect("Did not get a facture item id after insert");

    // Insert statut
    StatutFixture::recording_out_date(facture_id, facture_item_id)
        .insert_one(&mut tx)
        .await
        .unwrap();

    tx.commit().await.expect("Unable to commit the transaction");

    let app = facture_router().with_state(make_state(pool).await);
    let request = get_request("/factures");

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = body_to_string(response.into_body()).await;

    if status != StatusCode::OK {
        eprintln!("Error response body: {}", body);
    }
    assert_eq!(status, StatusCode::OK);

    assert!(body.contains("INV-001")); // Check for paper_ref from fixture
    assert_snapshot!("list_factures_with_data", body);
}
