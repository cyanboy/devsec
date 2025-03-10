use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tabled::Tabled;
use time::OffsetDateTime;

use crate::utils::repositories::display_offset_datetime;

#[derive(Tabled, Serialize, Deserialize, Debug)]
pub struct Repository {
    #[tabled(skip)]
    pub id: i64,

    #[tabled(skip)]
    pub external_id: i64,

    #[tabled(skip)]
    pub source: String,

    pub path: String,

    #[tabled(skip)]
    pub web_url: String,

    #[tabled(skip)]
    pub description: Option<String>,

    #[serde(with = "time::serde::rfc3339")]
    #[tabled(display("display_offset_datetime"))]
    pub created_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    #[tabled(skip)]
    pub updated_at: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    #[tabled(display("display_offset_datetime"))]
    pub pushed_at: OffsetDateTime,

    #[tabled(skip)]
    pub ssh_url: String,

    pub size: i64,
    pub commit_count: i64,

    #[tabled(skip)]
    pub forks_count: i64,

    #[tabled(skip)]
    pub private: bool,
    pub archived: bool,
}

#[derive(Debug)]
pub struct NewRepository {
    pub external_id: i64,
    pub source: String,
    pub path: String,
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
    pub language_name: String,
}

#[derive(Debug)]
pub struct RepositoryLanguage {
    pub repository_id: i64,
    pub language_id: i64,
    pub percentage: f64,
}

pub async fn insert_repository(
    pool: &SqlitePool,
    repository: NewRepository,
) -> Result<Repository, sqlx::Error> {
    let repo: Repository = sqlx::query_as!(
        Repository,
        r#"
            INSERT INTO repositories
            (
                external_id,
                source,
                path,
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
            VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
            ON CONFLICT (external_id, source) DO UPDATE
            SET
                path = excluded.path,
                source = excluded.source,
                description = excluded.description,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                pushed_at = excluded.pushed_at,
                ssh_url = excluded.ssh_url,
                web_url = excluded.web_url,
                private = excluded.private,
                forks_count = excluded.forks_count,
                archived = excluded.archived,
                size = excluded.size,
                commit_count = excluded.commit_count
            RETURNING
                id,
                external_id,
                source,
                path,
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
        repository.path,
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
    .fetch_one(pool)
    .await?;

    Ok(repo)
}

pub async fn insert_language(pool: &SqlitePool, name: &str) -> Result<Language, sqlx::Error> {
    let language: Language = sqlx::query_as!(
        Language,
        r#"
        INSERT INTO languages (language_name)
        VALUES (?)
        ON CONFLICT (language_name) DO UPDATE SET language_name = excluded.language_name
        RETURNING id, language_name
        "#,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(language)
}

pub async fn insert_repository_language(
    pool: &SqlitePool,
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
    .fetch_one(pool)
    .await?;

    Ok(repository_language)
}

pub async fn find_repository_by_path(pool: &SqlitePool, path: &str) -> Option<Repository> {
    sqlx::query_as!(
        Repository,
        r#"
        SELECT
            repo.id,
            repo.external_id,
            repo.source,
            repo.path,
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
        WHERE repo.path = ?
        "#,
        path
    )
    .fetch_optional(pool)
    .await
    .expect("Something unexpected while fetching repo by path")
}

pub async fn search_repositories(
    pool: &SqlitePool,
    query: &str,
    include_archived: bool,
    limit: i64,
) -> Result<Vec<Repository>, sqlx::Error> {
    let results: Vec<Repository> = sqlx::query_as!(
        Repository,
        r#"
        SELECT
            repo.id,
            repo.external_id,
            repo.source,
            repo.path,
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
