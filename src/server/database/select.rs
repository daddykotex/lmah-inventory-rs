use anyhow::{Context, Result};

use crate::server::{
    database::has_table::{HasTable, TableName},
    models::{
        clients::ClientRow,
        events::EventRow,
        facture_items::{FactureItemRow, ItemFactureFlowType},
        factures::FactureRow,
        statuts::StatutRow,
    },
};

pub trait Selectable<T> {
    fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<Option<T>>>;

    fn select_all(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<Vec<T>>>;
}

impl Selectable<ClientRow> for ClientRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<ClientRow>> {
        let table = ClientRow::table();
        let result: Option<ClientRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!("Failed to retrieve one client with id {}", id))?;

        Ok(result)
    }

    async fn select_all(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<ClientRow>> {
        let table = ClientRow::table();
        let result: Vec<ClientRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(&mut **tx)
                .await
                .context("Failed to retrieve clients")?;

        Ok(result)
    }
}

impl Selectable<EventRow> for EventRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<EventRow>> {
        let table = EventRow::table();
        let result: Option<EventRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!("Failed to retrieve one client with id {}", id))?;

        Ok(result)
    }

    async fn select_all(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<EventRow>> {
        let table = EventRow::table();
        let result: Vec<EventRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(&mut **tx)
                .await
                .context("Failed to retrieve events")?;

        Ok(result)
    }
}

impl Selectable<FactureRow> for FactureRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<FactureRow>> {
        let table = FactureRow::table();
        let result: Option<FactureRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!("Failed to retrieve one facture with id {}", id))?;

        Ok(result)
    }

    async fn select_all(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<FactureRow>> {
        let table = FactureRow::table();
        let result: Vec<FactureRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(&mut **tx)
                .await
                .context("Failed to retrieve factures")?;

        Ok(result)
    }
}

impl Selectable<FactureItemRow> for FactureItemRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<FactureItemRow>> {
        let table = FactureItemRow::table();
        let result: Option<FactureItemRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!(
            "Failed to retrieve one facture item with id {}",
            id
        ))?;

        Ok(result)
    }

    async fn select_all(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Vec<FactureItemRow>> {
        let table = FactureItemRow::table();
        let result: Vec<FactureItemRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} ORDER BY facture_id DESC, id DESC",
            table.table_name()
        ))
        .fetch_all(&mut **tx)
        .await
        .context("Failed to retrieve facture items")?;

        Ok(result)
    }
}

impl Selectable<StatutRow> for StatutRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<StatutRow>> {
        let table = StatutRow::table();
        let result: Option<StatutRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!("Failed to retrieve one statut with id {}", id))?;

        Ok(result)
    }

    async fn select_all(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<StatutRow>> {
        let table = StatutRow::table();
        let result: Vec<StatutRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} ORDER BY facture_id DESC, facture_item_id DESC",
            table.table_name()
        ))
        .fetch_all(&mut **tx)
        .await
        .context("Failed to retrieve statuts")?;

        Ok(result)
    }
}

impl ItemFactureFlowType {
    pub async fn select_all_flow_types(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Vec<ItemFactureFlowType>> {
        let result: Vec<ItemFactureFlowType> = sqlx::query_as(
            r#"
            select
                facture_items.facture_id as facture_id,
                facture_items.id as facture_item_id,
                case item_type
                    when 'Alteration' then 'AlterationFlow'
                    when 'Location' then 'LocationFlow'
                else case when product_types.name not in ('Robe de mariée', 'Robe de mère de la mariée', 'Robe de bal', 'Robe de bouquetière')
                        then 'AccessoryItemFlow'
                        else case when facture_items.floor_item
                                then 'DressFloorItemFlow'
                                else 'DressToOrderFlow'
                            end
                    end
                end as flow_type
            from facture_items
            left join products on facture_items.product_id=products.id
            left join product_product_types on products.id=product_product_types.product_id
            left join product_types on product_types.name=product_product_types.product_type_name
            order by facture_id DESC, facture_items.id DESC;
            "#
        )
        .fetch_all(&mut **tx)
        .await
        .context("Failed to facture item flows")?;

        Ok(result)
    }
}

impl ClientRow {
    pub async fn select_with_facture(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Vec<ClientRow>> {
        let facture_table = FactureRow::table();
        let client_table = ClientRow::table();
        let result: Vec<ClientRow> = sqlx::query_as(&format!(
            "SELECT {}.* FROM {} LEFT JOIN {} ON {}.client_id = {}.id",
            client_table.table_name(),
            facture_table.table_name(),
            client_table.table_name(),
            facture_table.table_name(),
            client_table.table_name()
        ))
        .fetch_all(&mut **tx)
        .await
        .context("Failed to retrieve clients")?;

        Ok(result)
    }
}
