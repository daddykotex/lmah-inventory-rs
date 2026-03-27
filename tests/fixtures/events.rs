use lmah_inventory_rs::server::models::events::EventInsert;

#[derive(Clone)]
pub struct EventFixture;

impl EventFixture {
    pub fn wedding() -> EventInsert {
        EventInsert {
            name: "Smith-Jones Wedding".to_string(),
            event_type: "Wedding".to_string(),
            date: "2026-06-15".to_string(),
        }
    }

    pub fn prom() -> EventInsert {
        EventInsert {
            name: "Spring Prom 2026".to_string(),
            event_type: "Prom".to_string(),
            date: "2026-05-20".to_string(),
        }
    }

    pub fn gala() -> EventInsert {
        EventInsert {
            name: "Charity Gala".to_string(),
            event_type: "Gala".to_string(),
            date: "2026-04-10".to_string(),
        }
    }
}
