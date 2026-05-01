//! Custom database queries that go beyond the basic Selectable trait.
//!
//! This module contains domain-specific queries with complex filtering,
//! joins, and aggregations. All queries use generic executor patterns
//! for maximum flexibility.

use anyhow::{Context, Result};
use futures_core::Stream;
use futures_util::StreamExt;
use sqlx::{Executor, Row, Sqlite, SqlitePool};

use crate::server::{
    database::has_table::{HasTable, TableName},
    models::{
        clients::ClientRow,
        facture_items::{FactureItemRow, ItemFactureFlowType},
        factures::FactureRow,
        payments::{PaymentReportRow, PaymentRow},
        product_types::ProductTypeRow,
        products::ProductRow,
        refunds::RefundRow,
        statuts::StatutRow,
    },
};

// === PAYMENT QUERIES ===
impl PaymentRow {
    pub async fn select_all_for_facture<'c, E>(
        facture_id: i64,
        executor: E,
    ) -> Result<Vec<PaymentRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = PaymentRow::table();
        let result: Vec<PaymentRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE facture_id = ?",
            table.table_name()
        ))
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context("Failed to retrieve payments")?;

        Ok(result)
    }

    pub async fn select_with_facture<'c, E>(executor: E) -> Result<Vec<PaymentRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<PaymentRow> = sqlx::query_as(
            "SELECT payments.* FROM payments LEFT JOIN factures ON factures.id = payments.facture_id ORDER BY factures.id",
        )
        .fetch_all(executor)
        .await
        .context("Failed to retrieve payments")?;

        Ok(result)
    }
}

impl PaymentReportRow {
    pub fn stream_all_with_facture(
        executor: &SqlitePool, // TODO figure out how to do it with a generic Executor
    ) -> impl Stream<Item = sqlx::Result<PaymentReportRow>> + 'static {
        executor
            .fetch(
                r#"
            SELECT
                payments.facture_id,
                factures.paper_ref,
                factures.facture_type,
                payments.date,
                payments.amount,
                payments.payment_type,
                factures.cancelled
            FROM payments
            LEFT JOIN factures ON payments.facture_id = factures.id
        "#,
            )
            .map(|row| {
                row.map(|row| PaymentReportRow {
                    facture_id: row.get("facture_id"),
                    paper_ref: row.get("paper_ref"),
                    facture_type: row.get("facture_type"),
                    date: row.get("date"),
                    amount: row.get("amount"),
                    payment_type: row.get("payment_type"),
                    cancelled: row.get("cancelled"),
                })
            })
    }
}

// === REFUND QUERIES ===

impl RefundRow {
    pub async fn select_all_for_facture<'c, E>(
        facture_id: i64,
        executor: E,
    ) -> Result<Vec<RefundRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = RefundRow::table();
        let result: Vec<RefundRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE facture_id = ?",
            table.table_name()
        ))
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context("Failed to retrieve refunds")?;

        Ok(result)
    }

    pub async fn select_with_facture<'c, E>(executor: E) -> Result<Vec<RefundRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<RefundRow> = sqlx::query_as(
            "SELECT refunds.* FROM refunds LEFT JOIN factures ON factures.id = refunds.facture_id ORDER BY factures.id",
        )
        .fetch_all(executor)
        .await
        .context("Failed to retrieve refunds")?;

        Ok(result)
    }
}

// === FACTURE QUERIES ===

impl FactureRow {
    pub async fn select_for_event<'c, E>(event_id: i64, executor: E) -> Result<Vec<FactureRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = FactureRow::table();
        let result: Vec<FactureRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE event_id = ?",
            table.table_name()
        ))
        .bind(event_id)
        .fetch_all(executor)
        .await
        .context(format!("Failed to retrieve factures with id {}", event_id))?;

        Ok(result)
    }
}

// === FACTURE ITEM QUERIES ===

impl FactureItemRow {
    pub async fn select_all_for_facture<'c, E>(
        facture_id: i64,
        executor: E,
    ) -> Result<Vec<FactureItemRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = FactureItemRow::table();
        let result: Vec<FactureItemRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE facture_id = ?",
            table.table_name()
        ))
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context(format!(
            "Failed to retrieve facture items for facture with id {}",
            facture_id
        ))?;

        Ok(result)
    }

    pub async fn select_with_facture<'c, E>(executor: E) -> Result<Vec<FactureItemRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<FactureItemRow> = sqlx::query_as(
            "SELECT facture_items.* FROM facture_items LEFT JOIN factures ON factures.id = facture_items.facture_id ORDER BY factures.id",
        )
        .fetch_all(executor)
        .await
        .context("Failed to retrieve facture_items")?;

        Ok(result)
    }
}

// === STATUT QUERIES ===

impl StatutRow {
    pub async fn select_all_for_facture<'c, E>(
        facture_id: i64,
        executor: E,
    ) -> Result<Vec<StatutRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = StatutRow::table();
        let result: Vec<StatutRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE facture_id = ? ORDER BY id ASC",
            table.table_name()
        ))
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context("Failed to retrieve statuts")?;

        Ok(result)
    }

    pub async fn select_all_for_facture_item<'c, E>(
        facture_item_id: i64,
        executor: E,
    ) -> Result<Vec<StatutRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = StatutRow::table();
        let result: Vec<StatutRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE facture_item_id = ? ORDER BY id ASC",
            table.table_name()
        ))
        .bind(facture_item_id)
        .fetch_all(executor)
        .await
        .context("Failed to retrieve statuts")?;

        Ok(result)
    }
}

// === PRODUCT QUERIES ===

impl ProductRow {
    pub async fn select_only_products<'c, E>(executor: E) -> Result<Vec<ProductRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ProductRow::table();
        let result: Vec<ProductRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE name NOT IN ('Altération', 'Location') ORDER BY id ASC",
            table.table_name()
        ))
        .fetch_all(executor)
        .await
        .context("Failed to retrieve all products but location and alteration")?;

        Ok(result)
    }

    pub async fn select_by_name<'c, E>(name: &str, executor: E) -> Result<Option<ProductRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ProductRow::table();
        let result: Option<ProductRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE name = ?",
            table.table_name()
        ))
        .bind(name)
        .fetch_optional(executor)
        .await
        .context(format!("Failed to retrieve one product with name {}", name))?;

        Ok(result)
    }

    pub async fn select_for_facture<'c, E>(facture_id: i64, executor: E) -> Result<Vec<ProductRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<ProductRow> = sqlx::query_as(
            "SELECT products.* FROM facture_items LEFT JOIN products ON facture_items.product_id = products.id WHERE facture_id = ?",
        )
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context(format!("Failed to retrieve products for facture id {}", facture_id))?;

        Ok(result)
    }
}

// === CLIENT QUERIES ===

impl ClientRow {
    pub async fn select_with_facture<'c, E>(executor: E) -> Result<Vec<ClientRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<ClientRow> = sqlx::query_as(
            "SELECT clients.* FROM clients LEFT JOIN factures ON factures.client_id = clients.id ORDER BY factures.id",
        )
        .fetch_all(executor)
        .await
        .context("Failed to retrieve clients")?;

        Ok(result)
    }

    pub async fn select_for_facture_event<'c, E>(
        event_id: i64,
        executor: E,
    ) -> Result<Vec<ClientRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<ClientRow> = sqlx::query_as(
            "SELECT clients.*
            FROM factures
            LEFT JOIN clients ON factures.client_id = clients.id
            WHERE factures.event_id = ?",
        )
        .bind(event_id)
        .fetch_all(executor)
        .await
        .context("Failed to retrieve clients for facture")?;

        Ok(result)
    }

    pub async fn select_all_for_facture<'c, E>(e: E) -> Result<Vec<ClientRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<ClientRow> =
            sqlx::query_as("SELECT clients.* FROM factures LEFT JOIN clients ON clients.id=factures.client_id ORDER by factures.id")
                .fetch_all(e)
                .await
                .context("Failed to retrieve clients")?;

        Ok(result)
    }
}

// === PRODUCT TYPE QUERIES ===

impl ProductTypeRow {
    pub async fn select_all<'c, E>(executor: E) -> Result<Vec<ProductTypeRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ProductTypeRow::table();
        let result: Vec<ProductTypeRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} ORDER BY name ASC",
            table.table_name()
        ))
        .fetch_all(executor)
        .await
        .context("Failed to retrieve product typ row")?;

        Ok(result)
    }

    pub async fn select_only_products<'c, E>(executor: E) -> Result<Vec<(i64, String)>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<(i64, String)> = sqlx::query_as(
            r#"
            SELECT product_id, product_type_name
            FROM product_product_types
            LEFT JOIN products ON products.id = product_product_types.product_id
            WHERE name NOT IN ('Altération', 'Location')
            ORDER BY product_id ASC
            "#,
        )
        .fetch_all(executor)
        .await
        .context("Failed to retrieve product_types for all products but location and alteration")?;

        Ok(result)
    }

    pub async fn select_for_facture<'c, E>(
        facture_id: i64,
        executor: E,
    ) -> Result<Vec<(i64, String)>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: Vec<(i64, String)> = sqlx::query_as(
            r#"
                SELECT product_product_types.product_id, product_product_types.product_type_name
                FROM facture_items
                LEFT JOIN products ON products.id = facture_items.product_id
                LEFT JOIN product_product_types ON products.id = product_product_types.product_id
                WHERE facture_items.facture_id = ?
                ORDER BY product_product_types.product_id ASC;
            "#,
        )
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context("Failed to retrieve product_types for all products but location and alteration")?;

        Ok(result)
    }

    pub async fn select_for_product<'c, E>(product_id: i64, executor: E) -> Result<ProductTypeRow>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: ProductTypeRow = sqlx::query_as(
            "SELECT name FROM product_types LEFT JOIN product_product_types ON product_types.name = product_product_types.product_type_name where product_id = ?",
        )
        .bind(product_id)
        .fetch_one(executor)
        .await?;

        Ok(result)
    }
}

// === ITEM FACTURE FLOW TYPE QUERIES ===

impl ItemFactureFlowType {
    pub async fn select_one_facture_item_flow_types<'c, E>(
        facture_item_id: i64,
        executor: E,
    ) -> Result<ItemFactureFlowType>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let result: ItemFactureFlowType = sqlx::query_as(
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
            where facture_items.id = ?
            order by facture_id DESC, facture_items.id DESC;
            "#
        )
        .bind(facture_item_id)
        .fetch_one(executor)
        .await
        .context("Failed to facture item flows")?;

        Ok(result)
    }

    pub async fn select_one_facture_flow_types<'c, E>(
        facture_id: i64,
        executor: E,
    ) -> Result<Vec<ItemFactureFlowType>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
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
            where facture_items.facture_id = ?
            order by facture_id DESC, facture_items.id DESC;
            "#
        )
        .bind(facture_id)
        .fetch_all(executor)
        .await
        .context("Failed to facture item flows")?;

        Ok(result)
    }

    pub async fn select_all_flow_types<'c, E>(executor: E) -> Result<Vec<ItemFactureFlowType>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
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
        .fetch_all(executor)
        .await
        .context("Failed to facture item flows")?;

        Ok(result)
    }
}
