use serde::Deserialize;
use sqlx::prelude::FromRow;

/// Database row structure for events table
#[derive(Debug, FromRow)]
pub struct EventRow {
    pub id: i64,
    pub name: String,
    pub event_type: String,
    pub date: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<EventForm> for EventInsert {
    fn from(value: EventForm) -> Self {
        EventInsert {
            name: value.name,
            event_type: value.event_type,
            date: value.date,
        }
    }
}

/// Database row structure for events table
#[derive(Debug)]
pub struct EventInsert {
    pub name: String,
    pub event_type: String,
    pub date: String,
}

/// Received from the UI, can be transformed into an EventInsert
#[derive(Deserialize, Debug)]
pub struct EventForm {
    pub name: String,
    pub date: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Used in the UI to display Event information
/// Very close if not the same to the EventRow
#[derive(Clone)]
pub struct EventView {
    pub id: i64,
    pub name: String,
    pub event_type: String,
    pub date: String,
}

impl From<EventRow> for EventView {
    fn from(value: EventRow) -> Self {
        EventView {
            id: value.id,
            name: value.name,
            event_type: value.event_type,
            date: value.date,
        }
    }
}
