use serde::{Deserialize, Serialize};
use tabled::Tabled;
use time::OffsetDateTime;

use crate::repositories::util::display_offset_datetime;

#[derive(Tabled, Serialize, Deserialize, Debug)]
pub struct Repository {
    #[tabled(skip)]
    pub id: i64,

    #[tabled(skip)]
    pub external_id: i64,

    #[tabled(skip)]
    pub source: String,

    #[tabled(skip)]
    pub name: String,

    #[tabled(skip)]
    pub namespace: String,

    #[tabled(rename = "url")]
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

    #[tabled(skip)]
    pub ssh_url: String,

    pub size: i64,
    pub commit_count: i64,
    pub forks_count: i64,
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct NewRepository {
    pub external_id: i64,
    pub source: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub pushed_at: OffsetDateTime,
    pub web_url: String,
    pub ssh_url: String,
    pub forks_count: i64,
    pub size: i64,
    pub commit_count: i64,
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct Language {
    pub id: i64,
    pub name: String,
}

#[derive(Debug)]
pub struct RepositoryLanguage {
    pub repository_id: i64,
    pub language_id: i64,
    pub percentage: f64,
}
