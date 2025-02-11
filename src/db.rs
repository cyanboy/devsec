use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct NewCodebase {
    pub external_id: i32,
    pub source: String,
    pub repo_name: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub ssh_url: String,
    pub web_url: String,
    pub private: bool,
    pub forks_count: i32,
    pub archived: bool,
    pub size: f32,
}

pub async fn insert_codebase(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    codebase: NewCodebase,
) -> Result<i32, sqlx::error::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO codebases 
        (external_id, source, repo_name, full_name, created_at, updated_at, pushed_at, ssh_url, web_url, private, forks_count, archived, size)
        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13 )
        ON CONFLICT (external_id, source) DO UPDATE 
        SET 
            repo_name = EXCLUDED.repo_name,
            full_name = EXCLUDED.full_name,
            created_at = EXCLUDED.created_at,
            updated_at = EXCLUDED.updated_at,
            pushed_at = EXCLUDED.pushed_at,
            ssh_url = EXCLUDED.ssh_url,
            web_url = EXCLUDED.web_url,
            private = EXCLUDED.private,
            forks_count = EXCLUDED.forks_count,
            archived = EXCLUDED.archived,
            size = EXCLUDED.size
        RETURNING id
        "#,
        codebase.external_id,
        codebase.source,
        codebase.repo_name,
        codebase.full_name,
        codebase.created_at,
        codebase.updated_at,
        codebase.pushed_at,
        codebase.ssh_url,
        codebase.web_url,
        codebase.private,
        codebase.forks_count,
        codebase.archived,
        codebase.size
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

pub async fn insert_codebase_language(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    codebase_id: i32,
    language_id: i32,
    percentage: f32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO codebase_languages (codebase_id, language_id, percentage)
        VALUES ($1, $2, $3) 
        ON CONFLICT (codebase_id, language_id) 
        DO UPDATE 
        SET percentage = EXCLUDED.percentage
        "#,
        codebase_id,
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
        FROM codebase_languages cl
        JOIN languages l ON cl.language_id = l.id
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
