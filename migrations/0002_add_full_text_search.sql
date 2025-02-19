CREATE VIRTUAL
TABLE repositories_fts USING fts5 (
    name,
    namespace,
    description,
    languages
);

INSERT INTO
    repositories_fts (
        rowid,
        name,
        namespace,
        description,
        languages
    )
SELECT r.id, r.name, r.namespace, r.description, COALESCE(
        (
            SELECT GROUP_CONCAT(l.language_name, ' ')
            FROM
                repository_languages rl
                JOIN languages l ON rl.language_id = l.id
            WHERE
                rl.repository_id = r.id
        ), ''
    )
FROM repositories r;

CREATE TRIGGER repositories_fts_insert AFTER INSERT ON repositories
BEGIN
    INSERT INTO repositories_fts (rowid, name, namespace, description, languages)
    VALUES (
        new.id,
        new.name,
        new.namespace,
        new.description,
        COALESCE((
            SELECT GROUP_CONCAT(l.language_name, ' ')
            FROM repository_languages rl
            JOIN languages l ON rl.language_id = l.id
            WHERE rl.repository_id = new.id
        ), '')
    );
END;

CREATE TRIGGER repositories_fts_update AFTER UPDATE ON repositories
BEGIN
    DELETE FROM repositories_fts WHERE rowid = old.id;
    INSERT INTO repositories_fts (rowid, name, namespace, description, languages)
    VALUES (
        new.id,
        new.name,
        new.namespace,
        new.description,
        COALESCE((
            SELECT GROUP_CONCAT(l.language_name, ' ')
            FROM repository_languages rl
            JOIN languages l ON rl.language_id = l.id
            WHERE rl.repository_id = new.id
        ), '')
    );
END;

CREATE TRIGGER repositories_fts_delete AFTER DELETE ON repositories
BEGIN
    DELETE FROM repositories_fts WHERE rowid = old.id;
END;

CREATE TRIGGER repository_languages_insert AFTER INSERT ON repository_languages
BEGIN
    UPDATE repositories_fts
    SET languages = COALESCE((
        SELECT GROUP_CONCAT(l.language_name, ' ')
        FROM repository_languages rl
        JOIN languages l ON rl.language_id = l.id
        WHERE rl.repository_id = new.repository_id
    ), '')
    WHERE rowid = new.repository_id;
END;

CREATE TRIGGER repository_languages_delete AFTER DELETE ON repository_languages
BEGIN
    UPDATE repositories_fts
    SET languages = COALESCE((
        SELECT GROUP_CONCAT(l.language_name, ' ')
        FROM repository_languages rl
        JOIN languages l ON rl.language_id = l.id
        WHERE rl.repository_id = old.repository_id
    ), '')
    WHERE rowid = old.repository_id;
END;