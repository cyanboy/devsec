use serde::Serialize;
use sqlx::SqlitePool;
use tabled::Tabled;

#[derive(Tabled, Debug, Serialize)]
pub struct RepoStats {
    pub total_repos: i64,
    pub largest_repo: String,
    pub most_active_repo: String,
    pub newest_repository: String,
    pub most_used_language: String,
    pub private_repo_count: i64,
    pub public_repo_count: i64,
}
pub async fn get_repository_statistics(pool: &SqlitePool) -> Result<RepoStats, sqlx::Error> {
    let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_repos,
                (SELECT path FROM codebases WHERE archived = FALSE ORDER BY size DESC LIMIT 1) as largest_repo,
                (SELECT path FROM codebases WHERE archived = FALSE ORDER BY commit_count DESC LIMIT 1) as most_active_repo
            FROM codebases
            WHERE archived = FALSE
            "#
        ).fetch_one(pool)
        .await?;

    let most_used_language = sqlx::query!(
        r#"
            SELECT
                programming_languages.name,
                COUNT(DISTINCT codebase_languages.language_id) as unique_languages
            FROM codebase_languages
            JOIN programming_languages ON codebase_languages.language_id = programming_languages.id
            JOIN codebases ON codebase_languages.codebase_id = codebases.id
            WHERE codebases.archived = FALSE
            GROUP BY programming_languages.name
            ORDER BY SUM(codebase_languages.percentage) DESC
            LIMIT 1
            "#
    )
    .fetch_optional(pool)
    .await?
    .map(|row| row.name)
    .unwrap_or_else(|| "Unknown".to_string());

    let private_repo_count = get_private_repo_count(pool).await?;
    let public_repo_count = get_public_repo_count(pool).await?;
    let newest_repository = get_newest_repo(pool).await?;

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

async fn get_private_repo_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT COUNT(*) private_count FROM codebases WHERE private = TRUE
        "#
    )
    .fetch_one(pool)
    .await?
    .private_count)
}

async fn get_public_repo_count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT COUNT(*) public_count FROM codebases WHERE private = FALSE
        "#
    )
    .fetch_one(pool)
    .await?
    .public_count)
}

async fn get_newest_repo(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT path FROM codebases WHERE archived = FALSE ORDER BY created_at DESC LIMIT 1;
        "#
    )
    .fetch_one(pool)
    .await?
    .path)
}
