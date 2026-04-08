use serde::Deserialize;
use sqlx::prelude::FromRow;

/// Database row structure for clients table
#[derive(Debug, FromRow, Clone)]
pub struct ClientRow {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Used to do an insert in the database, omit fields like created_at and updated_ad
#[derive(Debug)]
pub struct ClientInsert {
    pub first_name: String,
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
}

impl From<ClientForm> for ClientInsert {
    fn from(value: ClientForm) -> Self {
        ClientInsert {
            first_name: value.first_name,
            last_name: value.last_name,
            street: value.street,
            city: value.city,
            phone1: value.phone1,
            phone2: value.phone2,
        }
    }
}

/// Received from the UI, can be transformed into an ClientInsert
#[derive(Deserialize, Debug)]
pub struct ClientForm {
    #[serde(rename = "firstname")]
    pub first_name: String,
    #[serde(rename = "lastname")]
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
}

/// Used in the UI to display Client information
/// Very close if not the same to the ClientRow
#[derive(Clone)]
pub struct ClientView {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ClientView {
    pub fn name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

impl From<ClientRow> for ClientView {
    fn from(value: ClientRow) -> Self {
        ClientView {
            id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            street: value.street,
            city: value.city,
            phone1: value.phone1,
            phone2: value.phone2,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
