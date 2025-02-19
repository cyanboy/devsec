use sqlx::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::error::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id INTEGER NOT NULL,
            source TEXT NOT NULL,
            name TEXT NOT NULL,
            namespace TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            pushed_at TEXT NOT NULL,
            ssh_url TEXT NOT NULL,
            web_url TEXT NOT NULL,
            private BOOLEAN NOT NULL CHECK (private IN (0, 1)) DEFAULT 0,
            forks_count INTEGER NOT NULL,
            archived BOOLEAN NOT NULL CHECK (archived IN (0, 1)) DEFAULT 0,
            size INTEGER NOT NULL,
            commit_count INTEGER NOT NULL,
            UNIQUE (external_id, source)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS languages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS repository_languages (
            repository_id INTEGER NOT NULL,
            language_id INTEGER NOT NULL,
            percentage REAL NOT NULL,
            PRIMARY KEY (repository_id, language_id),
            FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
            FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS repositories_fts USING fts5 (
            name,
            namespace,
            description,
            languages
        );
      "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM repositories_fts WHERE rowid IN (SELECT id FROM repositories);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repositories_fts (
            rowid, name, namespace, description, languages
        )
        SELECT
            r.id,
            r.name,
            r.namespace,
            r.description,
            COALESCE((
                SELECT GROUP_CONCAT(l.name, ' ')
                FROM repository_languages rl
                JOIN languages l ON rl.language_id = l.id
                WHERE rl.repository_id = r.id
            ), '') AS languages
        FROM repositories r;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS repositories_fts_insert
        AFTER INSERT ON repositories
        BEGIN
            INSERT INTO repositories_fts (rowid, name, namespace, description, languages)
            VALUES (
                NEW.id,
                NEW.name,
                NEW.namespace,
                NEW.description,
                COALESCE((
                    SELECT GROUP_CONCAT(l.name, ' ')
                    FROM repository_languages rl
                    JOIN languages l ON rl.language_id = l.id
                    WHERE rl.repository_id = NEW.id
                ), '')
            );
        END
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS repositories_fts_update
        AFTER UPDATE ON repositories
        BEGIN
            DELETE FROM repositories_fts WHERE rowid = OLD.id;

            INSERT INTO repositories_fts (rowid, name, namespace, description, languages)
            VALUES (
                NEW.id,
                NEW.name,
                NEW.namespace,
                NEW.description,
                COALESCE((
                    SELECT GROUP_CONCAT(l.name, ' ')
                    FROM repository_languages rl
                    JOIN languages l ON rl.language_id = l.id
                    WHERE rl.repository_id = NEW.id
                ), '')
            );
        END;
    "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS repositories_fts_delete
        AFTER DELETE ON repositories
        BEGIN
            DELETE FROM repositories_fts WHERE rowid = OLD.id;
        END;

        CREATE TRIGGER IF NOT EXISTS repository_languages_insert
        AFTER INSERT ON repository_languages
        BEGIN
            UPDATE repositories_fts
            SET languages = COALESCE((
                SELECT GROUP_CONCAT(l.name, ' ')
                FROM repository_languages rl
                JOIN languages l ON rl.language_id = l.id
                WHERE rl.repository_id = NEW.repository_id
            ), '')
            WHERE rowid = NEW.repository_id;
        END;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER IF NOT EXISTS repository_languages_delete
        AFTER DELETE ON repository_languages
        BEGIN
            UPDATE repositories_fts
            SET languages = COALESCE((
                SELECT GROUP_CONCAT(l.name, ' ')
                FROM repository_languages rl
                JOIN languages l ON rl.language_id = l.id
                WHERE rl.repository_id = OLD.repository_id
            ), '')
            WHERE rowid = OLD.repository_id;
        END;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
