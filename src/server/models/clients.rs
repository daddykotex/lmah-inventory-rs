/// Database row structure for clients table
#[derive(Debug)]
pub struct ClientRow {
    pub airtable_id: String,
    pub first_name: String,
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
