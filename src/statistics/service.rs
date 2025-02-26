use sqlx::SqlitePool;

use crate::statistics::models::RepoStats;

pub struct StatisticsService<'a> {
    pool: &'a SqlitePool,
}

impl<'a> StatisticsService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_repository_statistics(&self) -> Result<RepoStats, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_repos,
                (SELECT name FROM repositories WHERE archived = FALSE ORDER BY size DESC LIMIT 1) as largest_repo,
                (SELECT name FROM repositories WHERE archived = FALSE ORDER BY commit_count DESC LIMIT 1) as most_active_repo
            FROM repositories
            WHERE archived = FALSE
            "#
        ).fetch_one(self.pool)
        .await?;

        let most_used_language = sqlx::query!(
            r#"
            SELECT
                languages.name,
                COUNT(DISTINCT repository_languages.language_id) as unique_languages
            FROM repository_languages
            JOIN languages ON repository_languages.language_id = languages.id
            JOIN repositories ON repository_languages.repository_id = repositories.id
            WHERE repositories.archived = FALSE
            GROUP BY languages.name
            ORDER BY SUM(repository_languages.percentage) DESC
            LIMIT 1
            "#
        )
        .fetch_optional(self.pool)
        .await?
        .map(|row| row.name)
        .unwrap_or_else(|| "Unknown".to_string());

        let private_repo_count = self.get_private_repo_count().await?;

        let public_repo_count = self.get_public_repo_count().await?;

        let newest_repository = self.get_newest_repo().await?;

        Ok(RepoStats {
            total_repos: row.total_repos,
            largest_repo: row.largest_repo.unwrap_or_default(),
            most_active_repo: row.most_active_repo.unwrap_or_default(),
            newest_repository,
            most_used_language,
            private_repo_count,
            public_repo_count,
        })
    }

    async fn get_private_repo_count(&self) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query!(
            r#"
            SELECT COUNT(*) private_count FROM repositories WHERE private = TRUE
            "#
        )
        .fetch_one(self.pool)
        .await?
        .private_count)
    }

    async fn get_public_repo_count(&self) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query!(
            r#"
            SELECT COUNT(*) public_count FROM repositories WHERE private = FALSE
            "#
        )
        .fetch_one(self.pool)
        .await?
        .public_count)
    }

    async fn get_newest_repo(&self) -> Result<String, sqlx::Error> {
        Ok(sqlx::query!(
            r#"
            SELECT name FROM repositories WHERE archived = FALSE ORDER BY created_at DESC LIMIT 1;
            "#
        )
        .fetch_one(self.pool)
        .await?
        .name)
    }
}
