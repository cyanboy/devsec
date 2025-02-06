use governor::{Jitter, Quota, RateLimiter};
use indicatif::ProgressBar;
use nonzero_ext::nonzero;
use reqwest::header::HeaderMap;
use sqlx::PgPool;
use std::time::Duration;
use std::{error::Error, sync::Arc};
use tokio::{sync::Mutex, task::JoinSet};

use crate::codebase::{
    get_codebases, get_codebases_without_languages, insert_codebase, insert_codebase_language,
    insert_language, Codebase,
};
use crate::gitlab::api::{Api, PER_PAGE_MAX, TOTAL_PAGES_HEADER};
use crate::progress_bar::style_progress_bar;

use super::api::GITLAB_PROJECT_RATE_LIMIT;

pub struct GitLabUpdater {
    api: Arc<Api>,
    pool: Arc<PgPool>,
    group_id: Option<String>,
}

impl GitLabUpdater {
    pub fn new(gitlab_token: &str, group_id: Option<String>, pool: PgPool) -> GitLabUpdater {
        let api = Arc::new(Api::new(&gitlab_token));
        let pool = Arc::new(pool.clone());

        GitLabUpdater {
            pool,
            group_id,
            api,
        }
    }

    pub async fn gitlab_update_projects(&self) -> Result<(), Box<dyn Error>> {
        let progress_bar = Arc::new(Mutex::new(ProgressBar::no_length()));

        let group_id = Arc::new(self.group_id.clone().expect("missing gitlab token"));

        let (projects, total_pages) = {
            let pb = progress_bar.lock().await;
            style_progress_bar(&pb);

            let (headers, projects) = self
                .api
                .groups_projects_get(&group_id, 1, PER_PAGE_MAX, true)
                .await?;

            let total_pages = parse_total_pages_header(&headers).unwrap_or(1);
            pb.set_length(total_pages as u64);
            pb.inc(1);

            (projects, total_pages)
        };

        let tx = self.pool.begin().await?;

        for project in projects {
            let repo = project.to_repository();
            insert_codebase(&self.pool, repo).await?;
        }

        let page_numbers: Vec<i32> = (2..=total_pages).collect();

        let mut set: JoinSet<Vec<Codebase>> = JoinSet::new();

        for page in page_numbers {
            let progress_bar = Arc::clone(&progress_bar);
            let api = Arc::clone(&self.api);
            let group_id = Arc::clone(&group_id);

            set.spawn(async move {
                let (_, projects) = api
                    .groups_projects_get(&group_id, page, PER_PAGE_MAX, true)
                    .await
                    .unwrap();

                progress_bar.lock().await.inc(1);
                projects.iter().map(|n| n.to_repository()).collect()
            });
        }

        let repositories = set.join_all().await;

        for repo in repositories.into_iter().flatten() {
            insert_codebase(&self.pool, repo)
                .await
                .expect("Failed to insert repository");
        }

        progress_bar.lock().await.finish_with_message("Completed!");
        tx.commit().await?;

        Ok(())
    }

    pub async fn gitlab_update_languages(&self) -> Result<(), Box<dyn Error>> {
        let bucket = Arc::new(RateLimiter::direct(Quota::per_minute(nonzero!(
            GITLAB_PROJECT_RATE_LIMIT
        ))));

        let codebases = get_codebases(&self.pool).await?;

        let progress_bar = ProgressBar::new(codebases.len() as u64);
        style_progress_bar(&progress_bar);

        let mut set = JoinSet::new();

        for codebase in codebases {
            let api = Arc::clone(&self.api);
            let bucket = Arc::clone(&bucket);

            set.spawn(async move {
                let jitter = Jitter::new(Duration::from_secs(1), Duration::from_secs(1));
                bucket.until_ready_with_jitter(jitter).await;
                let languages = api.gitlab_languages_get(codebase.id).await;
                (codebase.id, languages)
            });
        }

        while let Some(res) = set.join_next().await {
            let (id, languages) = res.unwrap();
            progress_bar.inc(1);

            for (lang, percent) in languages.unwrap() {
                let language_id = insert_language(&self.pool, &lang).await?;
                insert_codebase_language(&self.pool, id, language_id, percent).await?;
            }
        }

        progress_bar.finish_with_message("Completed!");

        Ok(())
    }
}

fn parse_total_pages_header(headers: &HeaderMap) -> Option<i32> {
    headers
        .get(TOTAL_PAGES_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i32>().ok())
}
