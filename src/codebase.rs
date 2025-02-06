use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct Codebase {
    pub id: i32,
    pub name: String,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub ssh_url: String,
    pub web_url: String,
    pub private: bool,
    pub forks_count: i32,
    pub default_branch: String,
    pub archived: bool,
}

pub async fn insert_codebase(pool: &PgPool, codebase: Codebase) -> Result<i32, sqlx::error::Error> {
    let rec = sqlx::query!(
        r#"INSERT INTO codebases 
        (id, name, full_name, created_at, updated_at, pushed_at, ssh_url, web_url, private, forks_count, default_branch, archived)
        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12 )
        ON CONFLICT (id) DO UPDATE 
        SET 
            name = EXCLUDED.name,
            full_name = EXCLUDED.full_name,
            created_at = EXCLUDED.created_at,
            updated_at = EXCLUDED.updated_at,
            pushed_at = EXCLUDED.pushed_at,
            ssh_url = EXCLUDED.ssh_url,
            web_url = EXCLUDED.web_url,
            private = EXCLUDED.private,
            forks_count = EXCLUDED.forks_count,
            default_branch = EXCLUDED.default_branch,
            archived = EXCLUDED.archived
        RETURNING id
        "#,
        codebase.id,
        codebase.name,
        codebase.full_name,
        codebase.created_at,
        codebase.updated_at,
        codebase.pushed_at,
        codebase.ssh_url,
        codebase.web_url,
        codebase.private,
        codebase.forks_count,
        codebase.default_branch,
        codebase.archived
    ).fetch_one(pool).await?;

    Ok(rec.id)
}

pub async fn get_codebases_without_languages(
    pool: &PgPool,
) -> Result<Vec<Codebase>, sqlx::error::Error> {
    let codebases = sqlx::query_as!(
        Codebase,
        r#"
        SELECT *
        FROM codebases c
        WHERE NOT EXISTS (
            SELECT 1 FROM codebase_languages cl
            WHERE cl.codebase_id = c.id
        )
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(codebases)
}

pub async fn get_codebases(pool: &PgPool) -> Result<Vec<Codebase>, sqlx::error::Error> {
    let codebases = sqlx::query_as!(
        Codebase,
        r#"
        SELECT *
        FROM codebases
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(codebases)
}

pub async fn insert_language(pool: &PgPool, language: &str) -> Result<i32, sqlx::error::Error> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO languages (name)
        VALUES ($1)
        ON CONFLICT (name) DO NOTHING
        RETURNING id
        "#,
        language
    )
    .fetch_optional(pool)
    .await?;

    Ok(match rec {
        Some(record) => record.id,
        None => {
            sqlx::query!("SELECT id FROM languages WHERE name = $1", language)
                .fetch_one(pool)
                .await?
                .id
        }
    })
}

pub async fn insert_codebase_language(
    pool: &PgPool,
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
    .execute(pool)
    .await?;

    Ok(())
}
