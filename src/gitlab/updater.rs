use std::error::Error;

use indicatif::ProgressBar;
use sqlx::SqlitePool;

use crate::{
    gitlab::api::{Api, Visibility},
    progress_bar::style_progress_bar,
    repositories::{insert_language, insert_repository, insert_repository_language, NewRepository},
};

pub struct GitLabUpdater {
    api: Api,
    group: String,
    pool: SqlitePool,
}

impl GitLabUpdater {
    pub fn new(gitlab_token: &str, group_id: &str, pool: SqlitePool) -> GitLabUpdater {
        let api = Api::new(&gitlab_token);
        let group = group_id.to_string();

        GitLabUpdater { api, group, pool }
    }

    pub async fn update(&self) -> Result<(), Box<dyn Error>> {
        let progress_bar = ProgressBar::no_length();

        let mut response = self
            .api
            .get_projects(&self.group)
            .await?
            .data
            .group
            .projects;

        style_progress_bar(&progress_bar);
        progress_bar.set_length(response.count);

        loop {
            let page_info = response.page_info;

            if let Some(cursor) = page_info.end_cursor {
                response = self
                    .api
                    .get_projects_after(&self.group, Some(&cursor))
                    .await?
                    .data
                    .group
                    .projects;
            } else {
                break;
            }

            for project in response.nodes {
                let (external_id, source) = {
                    let parts: Vec<&str> = project.id.split('/').collect();
                    assert_eq!(parts.len(), 5);

                    let src = parts[2].to_string();
                    let id: i64 = parts[4].parse()?;

                    (id, src)
                };

                let private = match project.visibility {
                    Visibility::Public => false,
                    _ => true,
                };

                let new_repository = NewRepository {
                    external_id,
                    source,
                    name: project.name,
                    namespace: project.namespace.full_path,
                    description: project.description,
                    created_at: project.created_at,
                    updated_at: project.updated_at,
                    pushed_at: project.last_activity_at,
                    ssh_url: project.ssh_url_to_repo,
                    web_url: project.web_url,
                    private,
                    forks_count: project.forks_count,
                    archived: project.archived,
                    size: project.statistics.repository_size as i64,
                    commit_count: project.statistics.commit_count as i64,
                };

                let mut tx = self.pool.begin().await?;

                let repo = insert_repository(&mut tx, new_repository).await?;

                for project_language in &project.languages {
                    let language = insert_language(&mut tx, &project_language.name).await?;
                    insert_repository_language(
                        &mut tx,
                        repo.id,
                        language.id,
                        project_language.share,
                    )
                    .await?;
                }

                tx.commit().await?;
                progress_bar.inc(1);
            }

            if !page_info.has_next_page {
                break;
            }
        }

        progress_bar.finish();
        Ok(())
    }
}
