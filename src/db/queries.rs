use sqlx::SqlitePool;

use crate::db::models::{NewRepository, Repository};

use super::models::Language;

pub async fn insert_repository(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    repository: NewRepository,
) -> Result<Repository, sqlx::error::Error> {
    let repo: Repository = sqlx::query_as(
        r#"
        INSERT INTO repositories
        (external_id, source, name, namespace, description, created_at, updated_at, pushed_at, ssh_url, web_url, private, forks_count, archived, size, commit_count)
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
        RETURNING *
        "#
    )
    .bind(repository.external_id)
    .bind(repository.source)
    .bind(repository.name)
    .bind(repository.namespace)
    .bind(repository.description)
    .bind(repository.created_at)
    .bind(repository.updated_at)
    .bind(repository.pushed_at)
    .bind(repository.ssh_url)
    .bind(repository.web_url)
    .bind(repository.private)
    .bind(repository.forks_count)
    .bind(repository.archived)
    .bind(repository.size)
    .bind(repository.commit_count)
    .fetch_one(&mut **transaction)
    .await?;

    Ok(repo)
}

pub async fn insert_language(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    name: &str,
) -> Result<Language, sqlx::error::Error> {
    let language: Option<Language> = sqlx::query_as(
        r#"
        INSERT INTO languages (name)
        VALUES (?)
        ON CONFLICT (name) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(name)
    .fetch_optional(&mut **transaction)
    .await?;

    let language = match language {
        Some(lang) => lang,
        None => {
            sqlx::query_as::<_, Language>("SELECT id, name FROM languages WHERE name = ?")
                .bind(name)
                .fetch_one(&mut **transaction)
                .await?
        }
    };

    Ok(language)
}

pub async fn insert_repository_language(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    repository_id: i64,
    language_id: i64,
    percentage: f32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO repository_languages (repository_id, language_id, percentage)
        VALUES (?, ?, ?)
        ON CONFLICT (repository_id, language_id)
        DO UPDATE
        SET percentage = excluded.percentage
        "#,
    )
    .bind(repository_id)
    .bind(language_id)
    .bind(percentage)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn get_most_frequent_languages(
    pool: &SqlitePool,
) -> Result<Vec<(String, f64)>, sqlx::error::Error> {
    let results: Vec<(String, f64)> = sqlx::query_as(
        r#"
        SELECT
        l.name,
        (SUM(cl.percentage) * 100.0) / SUM(SUM(cl.percentage)) OVER () AS usage
        FROM repository_languages cl
        JOIN languages l ON cl.language_id = l.id
        JOIN repositories c ON cl.repository_id = c.id
        WHERE c.archived = FALSE  -- Exclude archived repositories
        GROUP BY l.name
        ORDER BY usage DESC;
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn search_repositories(
    pool: &SqlitePool,
    query: &str,
    include_archived: bool,
) -> Result<Vec<Repository>, sqlx::error::Error> {
    let results: Vec<Repository> = sqlx::query_as(
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
        "#,
    )
    .bind(query)
    .bind(include_archived)
    .fetch_all(pool)
    .await?;

    Ok(results)
}
