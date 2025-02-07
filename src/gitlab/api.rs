use chrono::{DateTime, Utc};
use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Jitter, Quota, RateLimiter,
};
use rand::Rng;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, num::NonZeroU32, time::Duration};

use crate::db::Codebase;

pub const TOTAL_PAGES_HEADER: &str = "x-total-pages";
pub const PER_PAGE_MAX: u8 = 100;

const GITLAB_URL: &str = "https://gitlab.com/api/v4";
const PROJECTS_LANGUAGES_RATE_LIMIT: u32 = 200;
const GROUPS_PROJECTS_RATE_LIMIT: u32 = 600;

pub struct Api {
    client: Client,
    languages_rate_limit:
        RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
    projects_rate_limit:
        RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
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

        let languages_rate_limit = Self::create_rate_limit(PROJECTS_LANGUAGES_RATE_LIMIT);
        let group_projects_rate_limit = Self::create_rate_limit(GROUPS_PROJECTS_RATE_LIMIT);

        Self {
            client,
            languages_rate_limit,
            projects_rate_limit: group_projects_rate_limit,
        }
    }

    fn create_rate_limit(
        rate_limit: u32,
    ) -> RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>> {
        let rate_limit = NonZeroU32::new(rate_limit).unwrap();

        RateLimiter::direct(Quota::per_minute(rate_limit))
    }

    pub async fn get_groups_projects(
        &self,
        group: &str,
        page: i32,
        per_page: u8,
        include_subgroups: bool,
    ) -> Result<(HeaderMap, Vec<Project>), reqwest::Error> {
        let projects_url = format!("{}/groups/{}/projects", GITLAB_URL, group);

        let page = page.to_string();
        let per_page = per_page.to_string();
        let include_subgroups = include_subgroups.to_string();

        let rate_limit = &self.projects_rate_limit;

        let jitter = {
            let min = Duration::from_millis(rand::rng().random_range(500..=1500));
            Jitter::new(min, min)
        };

        rate_limit.until_ready_with_jitter(jitter).await;

        let response = self
            .client
            .get(projects_url)
            .query(&[
                ("page", page),
                ("per_page", per_page),
                ("include_subgroups", include_subgroups),
            ])
            .send()
            .await?;

        let headers = response.headers().clone();
        let projects = response.json::<Vec<Project>>().await?;

        Ok((headers, projects))
    }

    pub async fn gitlab_languages_get(
        &self,
        project_id: i32,
    ) -> Result<HashMap<String, f32>, reqwest::Error> {
        let languages_url = format!("{}/projects/{}/languages", GITLAB_URL, project_id);

        let rate_limit = &self.languages_rate_limit;

        let jitter = {
            let min = Duration::from_millis(rand::rng().random_range(500..=1500));
            Jitter::new(min, min)
        };

        rate_limit.until_ready_with_jitter(jitter).await;

        self.client
            .get(languages_url)
            .send()
            .await?
            .json::<HashMap<String, f32>>()
            .await
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub web_url: String,
    pub ssh_url_to_repo: String,
    pub forks_count: i32,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub path_with_namespace: String,
    pub archived: bool,
    pub visibility: Visibility,
    pub default_branch: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

impl Project {
    pub fn to_repository(&self) -> Codebase {
        Codebase {
            id: self.id,
            repo_name: self.name.clone(),
            full_name: self.path_with_namespace.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            pushed_at: self.last_activity_at,
            ssh_url: self.ssh_url_to_repo.clone(),
            web_url: self.web_url.clone(),
            private: match self.visibility {
                Visibility::Public => false,
                _ => true,
            },
            forks_count: self.forks_count,
            default_branch: self.default_branch.clone(),
            archived: self.archived,
        }
    }
}
