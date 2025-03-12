use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    domain::repository::{Codebase, NewCodebase},
    error::AppError,
    infrastructure::{
        api::gitlab::client::{
            GitLabClient,
            model::{Project, Visibility},
        },
        utils::progress_bar::style_progress_bar,
    },
    repository::codebase_repository::CodebaseRepository,
};

pub struct CodebaseService {
    codebase_repository: Box<dyn CodebaseRepository>,
    gitlab_client: GitLabClient,
}

impl CodebaseService {
    pub fn new(
        codebase_repository: Box<dyn CodebaseRepository>,
        gitlab_client: GitLabClient,
    ) -> Self {
        Self {
            codebase_repository,
            gitlab_client,
        }
    }

    pub async fn update_from_gitlab(&self, group_id: &str) -> Result<(), AppError> {
        let progress_bar = ProgressBar::new_spinner();
        style_progress_bar(&progress_bar);

        let mut cursor = None;
        let mut total_processed = 0;

        loop {
            let response = self
                .gitlab_client
                .get_projects_after(group_id, cursor.as_deref())
                .await?;

            let projects = response.data.group.projects;
            let page_info = projects.page_info;

            // Update progress bar if we know the total
            if total_processed == 0 && projects.count > 0 {
                progress_bar.set_length(projects.count as u64);
                progress_bar.set_style(ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})"
            ).unwrap());
            }

            for project in projects.nodes {
                // Process project
                self.process_project(project).await?;
                total_processed += 1;
                progress_bar.set_position(total_processed);
            }

            cursor = page_info.end_cursor;
            if !page_info.has_next_page || cursor.is_none() {
                break;
            }
        }

        progress_bar.finish_with_message(format!("Processed {} repositories", total_processed));
        Ok(())
    }

    async fn process_project(&self, project: Project) -> Result<(), AppError> {
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
