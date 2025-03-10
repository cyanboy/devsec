use model::GroupProjectsResponse;
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde_json::json;

const GITLAB_GRAPHQL_URL: &str = "https://gitlab.com/api/graphql";

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
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn get_projects_after(
        &self,
        group: &str,
        after: Option<&str>,
    ) -> Result<GroupProjectsResponse, reqwest::Error> {
        let query = r#"
            query GetGroupProjects($group_id: ID!, $after: String) {
                group(fullPath: $group_id) {
                    projects(includeSubgroups: true, after: $after) {
                        count
                        pageInfo {
                            endCursor
                            hasNextPage
                        }
                        nodes {
                            id
                            fullPath
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
                        }
                    }
                }
            }
        "#;

        let variables = if let Some(after) = after {
            json!({ "group_id": group, "after": after })
        } else {
            json!({ "group_id": group })
        };

        let data = json!({ "query": query, "variables": variables });

        let response = self
            .client
            .post(GITLAB_GRAPHQL_URL)
            .header(CONTENT_TYPE, "application/json")
            .json(&data)
            .send()
            .await?;

        let json = response.json::<GroupProjectsResponse>().await?;
        Ok(json)
    }

    pub async fn get_projects(&self, group: &str) -> Result<GroupProjectsResponse, reqwest::Error> {
        self.get_projects_after(group, None).await
    }
}

pub mod model {
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupProjectsResponse {
        pub data: GroupData,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GroupData {
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

    #[derive(Deserialize, Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Project {
        pub id: String,
        pub full_path: String,
        pub description: Option<String>,
        pub web_url: String,
        pub ssh_url_to_repo: String,
        pub forks_count: i64,
        #[serde(with = "time::serde::rfc3339")]
        pub created_at: OffsetDateTime,
        #[serde(with = "time::serde::rfc3339")]
        pub last_activity_at: OffsetDateTime,
        #[serde(with = "time::serde::rfc3339")]
        pub updated_at: OffsetDateTime,
        pub archived: bool,
        pub visibility: Visibility,
        pub languages: Vec<RepositoryLanguage>,
        pub statistics: ProjectStatistics,
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

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct PageInfo {
        pub end_cursor: Option<String>,
        pub has_next_page: bool,
    }
}
