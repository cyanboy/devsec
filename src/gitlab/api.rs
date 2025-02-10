use chrono::{DateTime, Utc};
use hyper::header::CONTENT_TYPE;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::Codebase;

const GITLAB_GRAPHQL_URL: &str = "https://gitlab.com/api/graphql";

const CODEBASES_QUERY: &str = r#"
query GetProjects($group: ID!, $after: String) { 
    group(fullPath: $group) { 
        projects(includeSubgroups: true, after: $after) {
            count
            pageInfo {
                endCursor
                hasNextPage
            }
            nodes {
                id
                name
                fullPath
                archived
                updatedAt
                createdAt
                lastActivityAt
                webUrl
                sshUrlToRepo
                forksCount
                visibility
                languages {
                    name
                    share
                }
            }
        }
    }
}"#;

pub struct Api {
    client: Client,
}

impl Api {
    pub fn new(token: &str) -> Self {
        let mut headers = HeaderMap::new();

        if let Ok(mut token) = HeaderValue::from_str(&format!("Bearer {token}")) {
            token.set_sensitive(true);
            headers.insert(AUTHORIZATION, token);
        } else {
            eprintln!("Could not set Authorization header");
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Could not create client");

        Self { client }
    }

    pub async fn get_projects(
        &self,
        group: &str,
        after: Option<&str>,
    ) -> Result<ProjectsResponse, reqwest::Error> {
        let data = json!({
            "query" : CODEBASES_QUERY,
            "variables" : {
                "group" : group,
                "after" : after.unwrap_or_default()
            }
        });

        self.client
            .post(GITLAB_GRAPHQL_URL)
            .header(CONTENT_TYPE, "application/json")
            .json(&data)
            .send()
            .await?
            .json::<ProjectsResponse>()
            .await
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub web_url: Option<String>,
    pub ssh_url_to_repo: Option<String>,
    pub forks_count: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub full_path: String,
    pub archived: Option<bool>,
    pub visibility: Option<Visibility>,
    pub languages: Vec<RepositoryLanguage>,
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

impl Project {
    pub fn to_repository(&self) -> Codebase {
        let id = self.id.split("/").last().unwrap().parse::<i32>().unwrap();

        Codebase {
            id,
            repo_name: self.name.clone(),
            full_name: self.full_path.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            pushed_at: self.last_activity_at,
            ssh_url: self.ssh_url_to_repo.clone(),
            web_url: self.web_url.clone(),
            private: match self.visibility {
                Some(Visibility::Public) => false,
                _ => true,
            },
            forks_count: self.forks_count,
            archived: self.archived,
        }
    }
}
