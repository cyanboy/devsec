use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, Transaction};
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

    #[tabled(skip)]
    pub name: String,

    #[tabled(skip)]
    pub namespace: String,

    #[tabled(rename = "url")]
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
    pub forks_count: i64,
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

pub struct RepositoryService<'a> {
    pool: &'a SqlitePool,
}

impl<'a> RepositoryService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_repository_and_verify(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        repository: NewRepository,
    ) -> Result<Repository, sqlx::Error> {
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
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        name: &str,
    ) -> Result<Language, sqlx::Error> {
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
        &self,
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

    pub async fn search_repositories(
        &self,
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
        .fetch_all(self.pool)
        .await?;

        Ok(results)
    }

    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Sqlite>, sqlx::Error> {
        self.pool.begin().await
    }

    pub async fn commit_transaction(
        &self,
        transaction: Transaction<'_, Sqlite>,
    ) -> Result<(), sqlx::Error> {
        transaction.commit().await
    }
}
