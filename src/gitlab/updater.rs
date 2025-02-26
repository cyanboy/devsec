use std::error::Error;

use indicatif::ProgressBar;

use crate::{
    gitlab::api::{Api, model::Visibility},
    repositories::{NewRepository, RepositoryService},
    utils::progress_bar::style_progress_bar,
};

pub struct GitLabUpdaterService<'a> {
    api: Api,
    group: String,
    repository_service: &'a RepositoryService<'a>,
}

impl<'a> GitLabUpdaterService<'a> {
    pub fn new(
        gitlab_token: &str,
        group_id: &str,
        repository_service: &'a RepositoryService,
    ) -> Self {
        let api = Api::new(gitlab_token);
        let group = group_id.to_string();

        Self {
            api,
            group,
            repository_service,
        }
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
                    private: !matches!(project.visibility, Visibility::Public),
                    forks_count: project.forks_count,
                    archived: project.archived,
                    size: project.statistics.repository_size as i64,
                    commit_count: project.statistics.commit_count as i64,
                };

                let mut tx = self.repository_service.begin_transaction().await?;

                let repo = self
                    .repository_service
                    .insert_repository_and_verify(&mut tx, new_repository)
                    .await?;

                for project_language in &project.languages {
                    let language = self
                        .repository_service
                        .insert_language_and_verify(&mut tx, &project_language.name)
                        .await?;

                    self.repository_service
                        .insert_repository_language_and_verify(
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
