use reqwest::header::HeaderMap;
use sqlx::PgPool;
use std::{error::Error, sync::Arc};
use tokio::{sync::Mutex, task::JoinSet};

use crate::codebase::{
    get_codebases, insert_codebase, insert_codebase_language, insert_language, Codebase,
};
use crate::gitlab::api::{Api, PER_PAGE_MAX, TOTAL_PAGES_HEADER};
use crate::progress_bar::create_progress_bar;

fn parse_total_pages_header(headers: &HeaderMap) -> Option<i32> {
    headers
        .get(TOTAL_PAGES_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<i32>().ok())
}

pub async fn gitlab_update_projects(
    auth: &str,
    group_id: &str,
    pool: &PgPool,
) -> Result<(), Box<dyn Error>> {
    let api = Arc::new(Api::new(&auth));
    let group_id = Arc::new(group_id.to_string());
    let pool = Arc::new(pool.clone());

    let (headers, projects) = api
        .groups_projects_get(&group_id, 1, PER_PAGE_MAX, true)
        .await?;

    let total_pages = parse_total_pages_header(&headers).unwrap_or(1);

    let progress_bar = Arc::new(Mutex::new(create_progress_bar(total_pages as u64)));
    let tx = pool.begin().await?;

    {
        let pb = progress_bar.lock().await;
        for project in projects {
            let repo = project.to_repository();
            insert_codebase(&pool, repo).await?;
        }
        pb.inc(1);
    }

    let page_numbers: Vec<i32> = (2..=total_pages).collect();

    let mut set: JoinSet<Vec<Codebase>> = JoinSet::new();

    for page in page_numbers {
        let group_id = Arc::clone(&group_id);
        let api = Arc::clone(&api);
        let progress_bar = Arc::clone(&progress_bar);

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
        insert_codebase(&pool, repo)
            .await
            .expect("Failed to insert repository");
    }

    progress_bar.lock().await.finish_with_message("Completed!");
    tx.commit().await?;

    Ok(())
}

pub async fn gitlab_update_languages(auth: &str, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let api = Arc::new(Api::new(&auth));
    let pool = Arc::new(pool.clone());

    let codebases = get_codebases(&pool).await?;

    let progress_bar = create_progress_bar(codebases.len() as u64);

    for codebase in codebases {
        let languages = api.gitlab_languages_get(codebase.id).await?;

        for (lang, percent) in languages {
            let language_id = insert_language(&pool, &lang).await?;
            insert_codebase_language(&pool, codebase.id, language_id, percent).await?;
        }
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("Completed!");

    Ok(())
}
