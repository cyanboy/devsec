use serde::{Deserialize, Serialize};
use tabled::Tabled;
use time::OffsetDateTime;

use crate::infrastructure::utils::repositories::display_offset_datetime;

#[derive(Tabled, Serialize, Deserialize, Debug)]
pub struct Codebase {
    #[tabled(skip)]
    pub id: i64,

    #[tabled(skip)]
    pub external_id: i64,

    #[tabled(skip)]
    pub source: String,

    pub path: String,

    #[tabled(skip)]
    pub web_url: String,

    #[tabled(skip)]
    pub description: Option<String>,

    #[serde(with = "time::serde::rfc3339")]
    #[tabled(display("display_offset_datetime"))]
    pub created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    #[tabled(skip)]
    pub updated_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    #[tabled(display("display_offset_datetime"))]
    pub pushed_at: OffsetDateTime,

    pub size: i64,
    pub commit_count: i64,

    #[tabled(skip)]
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct NewCodebase {
    pub external_id: i64,
    pub source: String,
    pub path: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub pushed_at: OffsetDateTime,
    pub web_url: String,
    pub size: i64,
    pub commit_count: i64,
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct ProgrammingLanguage {
    pub id: i64,
    pub name: String,
}

#[derive(Debug)]
pub struct CodebaseLanguage {
    pub codebase_id: i64,
    pub language_id: i64,
    pub percentage: f64,
}
