use indicatif::ProgressBar;

use crate::{
    domain::repository::{Codebase, NewCodebase},
    error::AppError,
    infrastructure::{
        api::gitlab::client::{GitLabClient, model::Visibility},
        utils::progress_bar::style_progress_bar,
    },
    repository::codebase_repository::CodebaseRepository,
};

pub struct CodebaseService<'a> {
    codebase_repository: CodebaseRepository<'a>,
    gitlab_client: GitLabClient,
}

impl<'a> CodebaseService<'a> {
    pub fn new(codebase_repository: CodebaseRepository<'a>, gitlab_client: GitLabClient) -> Self {
        Self {
            codebase_repository,
            gitlab_client,
        }
    }

    pub async fn update_from_gitlab(&self, group_id: &str) -> Result<(), AppError> {
        let progress_bar = ProgressBar::no_length();

        let mut response = self
            .gitlab_client
            .get_projects(group_id)
            .await?
            .data
            .group
            .projects;

        style_progress_bar(&progress_bar);
        progress_bar.set_length(response.count as u64);

        loop {
            let page_info = response.page_info;

            if let Some(cursor) = page_info.end_cursor {
                response = self
                    .gitlab_client
                    .get_projects_after(&group_id, Some(&cursor))
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
                    let id: i64 = parts[4].parse().unwrap();

                    (id, src)
                };

                let new_codebase = NewCodebase {
                    external_id,
                    source,
                    path: project.full_path,
                    description: project.description,
                    created_at: project.created_at,
                    updated_at: project.updated_at,
                    pushed_at: project.last_activity_at,
                    web_url: project.web_url,
                    private: !matches!(project.visibility, Visibility::Public),
                    archived: project.archived,
                    size: project.statistics.repository_size as i64,
                    commit_count: project.statistics.commit_count as i64,
                };

                let codebase = self.codebase_repository.save(new_codebase).await?;

                for project_language in &project.languages {
                    let lang = (project_language.name.as_str(), project_language.share);

                    self.codebase_repository
                        .add_language(&codebase, lang)
                        .await?;
                }

                progress_bar.inc(1);
            }

            if !page_info.has_next_page {
                break;
            }
        }

        progress_bar.finish();
        Ok(())
    }

    pub async fn search(
        &self,
        query: &str,
        include_archived: bool,
        limit: i64,
    ) -> Result<Vec<Codebase>, AppError> {
        self.codebase_repository
            .search(query, include_archived, limit)
            .await
            .map_err(AppError::Database)
    }
}
