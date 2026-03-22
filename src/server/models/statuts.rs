/// Database row structure for statuts table
#[derive(Debug)]
pub struct StatutRow {
    pub facture_id: i64,      // Required FK to factures
    pub facture_item_id: i64, // Required FK to facture_items
    pub statut_type: String,  // Type of status
    pub date: String,
    pub seamstress: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
