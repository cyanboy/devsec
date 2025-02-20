use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub namespace: Namespace,
    pub web_url: String,
    pub ssh_url_to_repo: String,
    pub forks_count: i64,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
    pub visibility: Visibility,
    pub languages: Vec<RepositoryLanguage>,
    pub statistics: ProjectStatistics,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Namespace {
    pub full_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectsResponse {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub group: Group,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub projects: ProjectConnection,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConnection {
    pub count: u64,
    pub page_info: PageInfo,
    pub nodes: Vec<Project>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepositoryLanguage {
    pub name: String,
    pub share: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatistics {
    pub repository_size: f32,
    pub commit_count: f32,
}
