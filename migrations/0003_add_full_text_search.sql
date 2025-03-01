CREATE VIRTUAL
TABLE repositories_fts USING fts5 (
    name, namespace, description, languages,
    tokenize = "unicode61 remove_diacritics 2"
);

INSERT INTO
    repositories_fts (
        rowid,
        name,
        namespace,
        description,
        languages
    )
SELECT r.id, r.name, r.namespace, r.description, IFNULL(
        (
            SELECT GROUP_CONCAT(
                    l.name, ' '
                    ORDER BY l.name
                )
            FROM
                repository_languages rl
                JOIN languages l ON rl.language_id = l.id
            WHERE
                rl.repository_id = r.id
        ), ''
    )
FROM repositories r;

CREATE TRIGGER repositories_fts_insert AFTER INSERT ON repositories BEGIN
    INSERT INTO repositories_fts (rowid, name, namespace, description, languages)
    VALUES (
        NEW.id,
        NEW.name,
        NEW.namespace,
        NEW.description,
        IFNULL((
            SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
            FROM repository_languages rl
            JOIN languages l ON rl.language_id = l.id
            WHERE rl.repository_id = NEW.id
        ), '')
    );
END;

CREATE TRIGGER repositories_fts_update AFTER UPDATE ON repositories BEGIN
    UPDATE repositories_fts
    SET
        name = NEW.name,
        namespace = NEW.namespace,
        description = NEW.description,
        languages = IFNULL((
            SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
            FROM repository_languages rl
            JOIN languages l ON rl.language_id = l.id
            WHERE rl.repository_id = NEW.id
        ), '')
    WHERE rowid = NEW.id;
END;

CREATE TRIGGER repositories_fts_delete AFTER DELETE ON repositories BEGIN
    DELETE FROM repositories_fts WHERE rowid = OLD.id;
END;

CREATE TRIGGER repository_languages_insert AFTER INSERT ON repository_languages BEGIN
    UPDATE repositories_fts
    SET languages = IFNULL((
        SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
        FROM repository_languages rl
        JOIN languages l ON rl.language_id = l.id
        WHERE rl.repository_id = NEW.repository_id
    ), '')
    WHERE rowid = NEW.repository_id;
END;

CREATE TRIGGER repository_languages_delete AFTER DELETE ON repository_languages BEGIN
    UPDATE repositories_fts
    SET languages = IFNULL((
        SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
        FROM repository_languages rl
        JOIN languages l ON rl.language_id = l.id
        WHERE rl.repository_id = OLD.repository_id
    ), '')
    WHERE rowid = OLD.repository_id;
END;