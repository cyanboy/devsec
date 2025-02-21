use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde_json::json;

use super::models::ProjectsResponse;

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

        match HeaderValue::from_str(&format!("Bearer {token}")) {
            Ok(mut token) => {
                token.set_sensitive(true);
                headers.insert(AUTHORIZATION, token);
            }
            _ => {
                eprintln!("Could not set Authorization header");
            }
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
