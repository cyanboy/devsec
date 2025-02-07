use std::fmt;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug)]
pub struct Codebase {
    pub id: i32,
    pub repo_name: String,
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
        (id, repo_name, full_name, created_at, updated_at, pushed_at, ssh_url, web_url, private, forks_count, default_branch, archived)
        VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12 )
        ON CONFLICT (id) DO UPDATE 
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
            default_branch = EXCLUDED.default_branch,
            archived = EXCLUDED.archived
        RETURNING id
        "#,
        codebase.id,
        codebase.repo_name,
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

pub async fn get_all_codebase_ids(pool: &PgPool) -> Result<Vec<i32>, sqlx::error::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id
        FROM codebases
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| r.id).collect())
}

pub async fn insert_language(
    pool: &PgPool,
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
    .fetch_optional(pool)
    .await?;

    Ok(match rec {
        Some(record) => record.id,
        None => {
            sqlx::query!(
                "SELECT id FROM languages WHERE language_name = $1",
                language_name
            )
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

pub struct Language {
    name: String,
    usage: f64,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}%", self.name, self.usage)
    }
}

pub async fn get_most_frequent_languages(
    pool: &PgPool,
) -> Result<Vec<Language>, sqlx::error::Error> {
    let results = sqlx::query!(
        r#"
        SELECT l.language_name AS language_name, 
        CAST(ROUND(
            CAST((SUM(cl.percentage) * 100.0) / SUM(SUM(cl.percentage)) OVER () AS NUMERIC), 
            5
        ) AS DOUBLE PRECISION) AS usage
        FROM codebase_languages cl
        JOIN languages l ON cl.language_id = l.id
        GROUP BY l.language_name
        ORDER BY usage DESC;
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|row| Language {
            name: row.language_name,
            usage: row.usage.unwrap_or(0.0),
        })
        .collect())
}
