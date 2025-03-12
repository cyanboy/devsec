use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::repository::{Codebase, CodebaseLanguage, NewCodebase, ProgrammingLanguage};

#[async_trait]
pub trait CodebaseRepository {
    async fn save(&self, new_codebase: NewCodebase) -> Result<Codebase, sqlx::Error>;
    async fn add_language(
        &self,
        codebase: &Codebase,
        lang: (&str, f64),
    ) -> Result<CodebaseLanguage, sqlx::Error>;
    async fn count(&self) -> Result<i64, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Codebase>, sqlx::Error>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Codebase>, sqlx::Error>;
    async fn search(
        &self,
        query: &str,
        include_archived: bool,
        limit: i64,
    ) -> Result<Vec<Codebase>, sqlx::Error>;
}

pub struct SqliteCodebaseRepository {
    pool: SqlitePool,
}

impl SqliteCodebaseRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CodebaseRepository for SqliteCodebaseRepository {
    async fn save(&self, new_codebase: NewCodebase) -> Result<Codebase, sqlx::Error> {
        sqlx::query_as!(
            Codebase,
            r#"
            INSERT INTO codebases
            (
                external_id,
                source,
                path,
                description,
                created_at,
                updated_at,
                pushed_at,
                web_url,
                private,
                archived,
                size,
                commit_count
            )
            VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
            ON CONFLICT (external_id, source) DO UPDATE
            SET
                path = excluded.path,
                source = excluded.source,
                description = excluded.description,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                pushed_at = excluded.pushed_at,
                web_url = excluded.web_url,
                private = excluded.private,
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
                web_url,
                private,
                archived,
                size,
                commit_count
            "#,
            new_codebase.external_id,
            new_codebase.source,
            new_codebase.path,
            new_codebase.description,
            new_codebase.created_at,
            new_codebase.updated_at,
            new_codebase.pushed_at,
            new_codebase.web_url,
            new_codebase.private,
            new_codebase.archived,
            new_codebase.size,
            new_codebase.commit_count,
        )
        .fetch_one(&self.pool)
        .await
    }

    async fn count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(r#"SELECT COUNT(id) as count FROM codebases"#)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.count)
    }

    async fn find_all(&self) -> Result<Vec<Codebase>, sqlx::Error> {
        sqlx::query_as!(
            Codebase,
            r#"
            SELECT
                id,
                external_id,
                source,
                path,
                description,
                created_at as "created_at: _",
                updated_at as "updated_at: _",
                pushed_at as "pushed_at: _",
                web_url,
                private,
                archived,
                size,
                commit_count
            FROM codebases
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Codebase>, sqlx::Error> {
        sqlx::query_as!(
            Codebase,
            r#"
            SELECT
                id,
                external_id,
                source,
                path,
                description,
                created_at as "created_at: _",
                updated_at as "updated_at: _",
                pushed_at as "pushed_at: _",
                web_url,
                private,
                archived,
                size,
                commit_count
            FROM codebases WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    async fn add_language(
        &self,
        codebase: &Codebase,
        lang: (&str, f64),
    ) -> Result<CodebaseLanguage, sqlx::Error> {
        let (name, percentage) = lang;

        let mut tx = self.pool.begin().await?;

        let language = sqlx::query_as!(
            ProgrammingLanguage,
            r#"
            INSERT INTO programming_languages (name)
            VALUES (?)
            ON CONFLICT (name)
            DO UPDATE
            SET
                name = excluded.name
            RETURNING id, name
            "#,
            name
        )
        .fetch_one(&mut *tx)
        .await?;

        let codebase_language = sqlx::query_as!(
            CodebaseLanguage,
            r#"
            INSERT INTO codebase_languages (codebase_id, language_id, percentage)
            VALUES (?, ?, ?)
            ON CONFLICT (codebase_id, language_id)
            DO UPDATE SET percentage = excluded.percentage
            RETURNING codebase_id, language_id, percentage
            "#,
            codebase.id,
            language.id,
            percentage,
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(codebase_language)
    }

    async fn search(
        &self,
        query: &str,
        include_archived: bool,
        limit: i64,
    ) -> Result<Vec<Codebase>, sqlx::Error> {
        let results: Vec<Codebase> = sqlx::query_as!(
            Codebase,
            r#"
            SELECT
                c.id,
                c.external_id,
                c.source,
                c.path,
                c.description,
                c.created_at as "created_at: _",
                c.updated_at as "updated_at: _",
                c.pushed_at as "pushed_at: _",
                c.web_url,
                c.private,
                c.archived,
                c.size,
                c.commit_count
            FROM codebases c
            JOIN codebases_fts ON codebases_fts.rowid = c.id
            WHERE codebases_fts LIKE '%' + ? + '%'
            AND (CASE WHEN ? THEN 1 ELSE c.archived = FALSE END)
            ORDER BY bm25(codebases_fts)
            LIMIT ?
            "#,
            query,
            include_archived,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}
