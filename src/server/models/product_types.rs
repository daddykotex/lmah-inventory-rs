/// Database row structure for product_types table
/// Note: name is the primary key, no airtable_id or timestamps needed
#[derive(Debug)]
pub struct ProductTypeRow {
    pub name: String,
}
