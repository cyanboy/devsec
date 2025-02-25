use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub id: i64,
    pub external_id: i64,
    pub source: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub pushed_at: OffsetDateTime,
    pub web_url: String,
    pub ssh_url: String,
    pub forks_count: i64,
    pub size: i64,
    pub commit_count: i64,
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct NewRepository {
    pub external_id: i64,
    pub source: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub pushed_at: OffsetDateTime,
    pub web_url: String,
    pub ssh_url: String,
    pub forks_count: i64,
    pub size: i64,
    pub commit_count: i64,
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct Language {
    pub id: i64,
    pub name: String,
}

#[derive(Debug)]
pub struct RepositoryLanguage {
    pub repository_id: i64,
    pub language_id: i64,
    pub percentage: f64,
}

pub async fn insert_repository_and_verify(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    repository: NewRepository,
) -> Result<Repository, sqlx::error::Error> {
    let repo: Repository = sqlx::query_as!(
        Repository,
        r#"
        INSERT INTO repositories
        (
            external_id,
            source,
            name,
            namespace,
            description,
            created_at,
            updated_at,
            pushed_at,
            ssh_url,
            web_url,
            private,
            forks_count,
            archived,
            size,
            commit_count
        )
        VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
        ON CONFLICT (external_id, source) DO UPDATE
        SET
            name = name,
            namespace = namespace,
            description = description,
            created_at = created_at,
            updated_at = updated_at,
            pushed_at = pushed_at,
            ssh_url = ssh_url,
            web_url = web_url,
            private = private,
            forks_count = forks_count,
            archived = archived,
            size = size,
            commit_count = commit_count
        RETURNING
            id,
            external_id,
            source,
            name,
            namespace,
            description,
            created_at as "created_at: _",
            updated_at as "updated_at: _",
            pushed_at as "pushed_at: _",
            ssh_url,
            web_url,
            private,
            forks_count,
            archived,
            size,
            commit_count
        "#,
        repository.external_id,
        repository.source,
        repository.name,
        repository.namespace,
        repository.description,
        repository.created_at,
        repository.updated_at,
        repository.pushed_at,
        repository.ssh_url,
        repository.web_url,
        repository.private,
        repository.forks_count,
        repository.archived,
        repository.size,
        repository.commit_count,
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(repo)
}

pub async fn insert_language_and_verify(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    name: &str,
) -> Result<Language, sqlx::error::Error> {
    let language: Language = sqlx::query_as!(
        Language,
        r#"
        INSERT INTO languages (name)
        VALUES (?)
        ON CONFLICT (name) DO UPDATE SET name = excluded.name
        RETURNING id, name
        "#,
        name
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(language)
}

pub async fn insert_repository_language_and_verify(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    repository_id: i64,
    language_id: i64,
    percentage: f32,
) -> Result<RepositoryLanguage, sqlx::Error> {
    let repository_language = sqlx::query_as!(
        RepositoryLanguage,
        r#"
        INSERT INTO repository_languages (repository_id, language_id, percentage)
        VALUES (?, ?, ?)
        ON CONFLICT (repository_id, language_id)
        DO UPDATE SET percentage = excluded.percentage
        RETURNING repository_id, language_id, percentage
        "#,
        repository_id,
        language_id,
        percentage,
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(repository_language)
}

pub async fn get_most_frequent_languages(
    pool: &SqlitePool,
) -> Result<Vec<(String, f64)>, sqlx::error::Error> {
    let results: Vec<(String, f64)> = sqlx::query!(
        r#"
        SELECT
        l.name,
        COALESCE((SUM(cl.percentage) * 100.0) / SUM(SUM(cl.percentage)) OVER (), 0.0) AS usage
        FROM repository_languages cl
        JOIN languages l ON cl.language_id = l.id
        JOIN repositories c ON cl.repository_id = c.id
        WHERE c.archived = FALSE  -- Exclude archived repositories
        GROUP BY l.name
        ORDER BY usage DESC;
        "#,
    )
    .map(|row| (row.name, row.usage))
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn search_repositories(
    pool: &SqlitePool,
    query: &str,
    include_archived: bool,
    limit: i64,
) -> Result<Vec<Repository>, sqlx::error::Error> {
    let results: Vec<Repository> = sqlx::query_as!(
        Repository,
        r#"
        SELECT
            repo.id,
            repo.external_id,
            repo.source,
            repo.name,
            repo.namespace,
            repo.description,
            repo.created_at as "created_at: _",
            repo.updated_at as "updated_at: _",
            repo.pushed_at as "pushed_at: _",
            repo.ssh_url,
            repo.web_url,
            repo.private,
            repo.forks_count,
            repo.archived,
            repo.size,
            repo.commit_count
        FROM repositories repo
        JOIN repositories_fts ON repositories_fts.rowid = repo.id
        WHERE repositories_fts MATCH ?
        AND (CASE WHEN ? THEN 1 ELSE repo.archived = FALSE END)
        ORDER BY bm25(repositories_fts)
        LIMIT ?
        "#,
        query,
        include_archived,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}
