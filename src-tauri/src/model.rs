use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Entry {
    pub timestamp: NaiveDateTime,
    pub content: String,
}
