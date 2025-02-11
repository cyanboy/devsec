use std::error::Error;

use indicatif::ProgressBar;
use sqlx::PgPool;

use crate::{
    db::{insert_codebase, insert_codebase_language, insert_language, NewCodebase},
    gitlab::api::{Api, Visibility},
    progress_bar::style_progress_bar,
};

pub struct GitLabUpdater {
    api: Api,
    group: String,
    pool: PgPool,
}

impl GitLabUpdater {
    pub fn new(gitlab_token: &str, group_id: &str, pool: PgPool) -> GitLabUpdater {
        let api = Api::new(&gitlab_token);
        let group = group_id.to_string();

        GitLabUpdater { api, group, pool }
    }

    pub async fn update(&self) -> Result<(), Box<dyn Error>> {
        let progress_bar = ProgressBar::no_length();

        let mut response = self.api.get_projects("ruter-as").await?.data.group.projects;

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
                    let id: i32 = parts[4].parse()?;

                    (id, src)
                };

                let private = match project.visibility {
                    Visibility::Public => false,
                    _ => true,
                };

                let codebase = NewCodebase {
                    external_id,
                    source,
                    repo_name: project.name,
                    full_name: project.full_path,
                    created_at: project.created_at,
                    updated_at: project.updated_at,
                    pushed_at: project.last_activity_at,
                    ssh_url: project.ssh_url_to_repo,
                    web_url: project.web_url,
                    private,
                    forks_count: project.forks_count,
                    archived: project.archived,
                    size: project.statistics.repository_size,
                };

                let mut tx = self.pool.begin().await?;

                let codebase_id = insert_codebase(&mut tx, codebase).await?;

                for lang in &project.languages {
                    let lang_id = insert_language(&mut tx, &lang.name).await?;
                    insert_codebase_language(&mut tx, codebase_id, lang_id, lang.share).await?;
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
