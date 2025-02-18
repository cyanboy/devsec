use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub id: i32,
    pub external_id: i32,
    pub source: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub ssh_url: String,
    pub web_url: String,
    pub private: bool,
    pub forks_count: i32,
    pub archived: bool,
    pub size: i64,
    pub commit_count: i32,
}

#[derive(Debug)]
pub struct NewRepository {
    pub external_id: i32,
    pub source: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub ssh_url: String,
    pub web_url: String,
    pub private: bool,
    pub forks_count: i32,
    pub archived: bool,
    pub size: i64,
    pub commit_count: i32,
}

pub async fn insert_repository(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    repository: NewRepository,
) -> Result<i32, sqlx::error::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO repositories
        (external_id, source, name, namespace, description, created_at, updated_at, pushed_at, ssh_url, web_url, private, forks_count, archived, size, commit_count)
        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
        ON CONFLICT (external_id, source) DO UPDATE
        SET
            name = EXCLUDED.name,
            namespace = EXCLUDED.namespace,
            description = EXCLUDED.description,
            created_at = EXCLUDED.created_at,
            updated_at = EXCLUDED.updated_at,
            pushed_at = EXCLUDED.pushed_at,
            ssh_url = EXCLUDED.ssh_url,
            web_url = EXCLUDED.web_url,
            private = EXCLUDED.private,
            forks_count = EXCLUDED.forks_count,
            archived = EXCLUDED.archived,
            size = EXCLUDED.size,
            commit_count = EXCLUDED.commit_count
        RETURNING id
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
    ).fetch_one(&mut **transaction).await?;

    Ok(rec.id)
}

pub async fn insert_language(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    language_name: &str,
) -> Result<i32, sqlx::error::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO languages (language_name)
        VALUES ($1)
        ON CONFLICT (language_name) DO NOTHING
        RETURNING id
        "#,
        language_name
    )
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(match rec {
        Some(record) => record.id,
        None => {
            sqlx::query!(
                "SELECT id FROM languages WHERE language_name = $1",
                language_name
            )
            .fetch_one(&mut **transaction)
            .await?
            .id
        }
    })
}

pub async fn insert_repository_language(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    repository_id: i32,
    language_id: i32,
    percentage: f32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO repository_languages (repository_id, language_id, percentage)
        VALUES ($1, $2, $3)
        ON CONFLICT (repository_id, language_id)
        DO UPDATE
        SET percentage = EXCLUDED.percentage
        "#,
        repository_id,
        language_id,
        percentage
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn get_most_frequent_languages(
    pool: &PgPool,
) -> Result<Vec<(String, f64)>, sqlx::error::Error> {
    let results = sqlx::query!(
        r#"
        SELECT
        l.language_name AS language_name,
        (SUM(cl.percentage) * 100.0) / SUM(SUM(cl.percentage)) OVER () AS usage
        FROM repository_languages cl
        JOIN languages l ON cl.language_id = l.id
        JOIN repositories c ON cl.repository_id = c.id
        WHERE c.archived = FALSE  -- Exclude archived repositories
        GROUP BY l.language_name
        ORDER BY usage DESC;
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .iter()
        .map(|row| (row.language_name.clone(), row.usage.unwrap_or(0.0)))
        .collect())
}

pub async fn search_repositories(
    pool: &PgPool,
    query: &str,
    include_archived: bool,
) -> Result<Vec<Repository>, sqlx::error::Error> {
    let results = if include_archived {
        sqlx::query_as!(
            Repository,
            r#"
            SELECT id, external_id, source, name, namespace, description, created_at, updated_at, pushed_at, ssh_url, web_url,
                private, forks_count, archived, size, commit_count
            FROM repositories
            WHERE search_vector @@ websearch_to_tsquery('english', $1)
            ORDER BY ts_rank(search_vector, websearch_to_tsquery('english', $1)) DESC;
            "#,
            query
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            Repository,
            r#"
            SELECT id, external_id, source, name, namespace, description, created_at, updated_at, pushed_at, ssh_url, web_url,
                private, forks_count, archived, size, commit_count
            FROM repositories
            WHERE search_vector @@ websearch_to_tsquery('english', $1)
            AND archived = false
            ORDER BY ts_rank(search_vector, websearch_to_tsquery('english', $1)) DESC;
            "#,
            query
        )
        .fetch_all(pool)
        .await?
    };

    Ok(results)
}
