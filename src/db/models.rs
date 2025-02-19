use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Repository {
    pub id: i64,
    pub external_id: i64,
    pub source: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub web_url: String,
    pub ssh_url: String,
    pub forks_count: i64,
    pub size: i64,
    pub commit_count: i64,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub web_url: String,
    pub ssh_url: String,
    pub forks_count: i64,
    pub size: i64,
    pub commit_count: i64,
    pub private: bool,
    pub archived: bool,
}

#[derive(FromRow, Debug)]
pub struct Language {
    pub id: i64,
    pub name: String,
}
