use crate::server::models::clients::{Client, ClientForm, ClientView};
use anyhow::{Context, Result};
use toasty::Db;

pub async fn insert_client(db: &mut Db, form: ClientForm) -> Result<u64> {
    let mut tx = db
        .transaction()
        .await
        .context("Failed to begin transaction")?;
    let client = toasty::create!(Client {
        first_name: form.first_name,
        last_name: form.last_name,
        street: form.street,
        city: form.city,
        phone1: form.phone1,
        phone2: form.phone2,
    })
    .exec(&mut tx)
    .await?;
    tx.commit().await.context("Failed to commit transaction")?;
    let client: ClientView = client.into();

    Ok(client.id)
}

pub async fn update_client(db: &mut Db, id: u64, form: ClientForm) -> Result<u64> {
    let mut tx = db
        .transaction()
        .await
        .context("Failed to begin transaction")?;
    Client::filter_by_id(id)
        .update()
        .first_name(form.first_name)
        .last_name(form.last_name)
        .street(form.street)
        .city(form.city)
        .phone1(form.phone1)
        .phone2(form.phone2)
        .exec(&mut tx)
        .await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(1) //TODO
}
