use lmah_inventory_rs::server::models::clients::ClientInsert;

#[derive(Clone)]
pub struct ClientFixture;

impl ClientFixture {
    pub fn alice() -> ClientInsert {
        ClientInsert {
            first_name: "Alice".to_string(),
            last_name: "Anderson".to_string(),
            street: Some("123 Maple St".to_string()),
            city: Some("Montreal".to_string()),
            phone1: "(123) 456-7890".to_string(),
            phone2: None,
        }
    }

    pub fn bob() -> ClientInsert {
        ClientInsert {
            first_name: "Bob".to_string(),
            last_name: "Brown".to_string(),
            street: None,
            city: None,
            phone1: "(234) 567-8901".to_string(),
            phone2: Some("(235) 567-8901".to_string()),
        }
    }

    pub fn charlie() -> ClientInsert {
        ClientInsert {
            first_name: "Charlie".to_string(),
            last_name: "Clark".to_string(),
            street: Some("789 Oak Ave".to_string()),
            city: Some("Quebec".to_string()),
            phone1: "(345) 678-9012".to_string(),
            phone2: None,
        }
    }
}
