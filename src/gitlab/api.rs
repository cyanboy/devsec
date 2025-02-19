use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const GITLAB_GRAPHQL_URL: &str = "https://gitlab.com/api/graphql";

const GET_PROJECTS_QUERY: &str = r#"
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
                description
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
                statistics {
                    repositorySize
                    commitCount
                }
                namespace {
                    fullPath
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

        let auth_header =
            HeaderValue::from_str(&format!("Bearer {token}")).expect("Invalid token format");

        headers.insert(AUTHORIZATION, auth_header);

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get_projects_after(
        &self,
        group: &str,
        after: Option<&str>,
    ) -> Result<ProjectsResponse, reqwest::Error> {
        let variables = if let Some(after) = after {
            json!({ "group": group, "after": after })
        } else {
            json!({ "group": group })
        };

        let data = json!({ "query": GET_PROJECTS_QUERY, "variables": variables });

        let response = self
            .client
            .post(GITLAB_GRAPHQL_URL)
            .header(CONTENT_TYPE, "application/json")
            .json(&data)
            .send()
            .await?;

        let json = response.json::<ProjectsResponse>().await.unwrap();
        Ok(json)
    }

    pub async fn get_projects(&self, group: &str) -> Result<ProjectsResponse, reqwest::Error> {
        self.get_projects_after(group, None).await
    }
}

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
