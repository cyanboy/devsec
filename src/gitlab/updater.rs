use std::error::Error;

use indicatif::ProgressBar;
use sqlx::PgPool;

use crate::{
    db::{insert_codebase, insert_codebase_language, insert_language},
    gitlab::api::Api,
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

        let mut response = self
            .api
            .get_projects("ruter-as", None)
            .await?
            .data
            .group
            .projects;

        style_progress_bar(&progress_bar);
        progress_bar.set_length(response.count);

        loop {
            let mut tx = self.pool.begin().await?;

            let page_info = response.page_info;

            if let Some(cursor) = page_info.end_cursor {
                response = self
                    .api
                    .get_projects(&self.group, Some(&cursor))
                    .await?
                    .data
                    .group
                    .projects;
            } else {
                break;
            }

            for project in response.nodes {
                let codebase = project.to_repository();
                let languages = project.languages;

                let codebase_id = codebase.id;

                insert_codebase(&mut tx, codebase).await?;

                for lang in languages {
                    let lang_id = insert_language(&mut tx, &lang.name).await?;
                    insert_codebase_language(&mut tx, codebase_id, lang_id, lang.share).await?;
                }

                progress_bar.inc(1);
            }

            tx.commit().await?;

            if !page_info.has_next_page {
                break;
            }
        }

        progress_bar.finish_with_message("Done updating projects!");
        Ok(())
    }
}
