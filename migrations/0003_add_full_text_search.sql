CREATE VIRTUAL
TABLE repositories_fts USING fts5 (
    path, description, languages,
    tokenize = "unicode61 remove_diacritics 2"
);

INSERT INTO
    repositories_fts (
        rowid,
        path,
        description,
        languages
    )
SELECT r.id, r.path, r.description, IFNULL(
        (
            SELECT GROUP_CONCAT(
                    l.language_name, ' '
                    ORDER BY l.language_name
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
    INSERT INTO repositories_fts (rowid, path, description, languages)
    VALUES (
        NEW.id,
        NEW.path,
        NEW.description,
        IFNULL((
            SELECT GROUP_CONCAT(l.language_name, ' ' ORDER BY l.language_name)
            FROM repository_languages rl
            JOIN languages l ON rl.language_id = l.id
            WHERE rl.repository_id = NEW.id
        ), '')
    );
END;

CREATE TRIGGER repositories_fts_update AFTER UPDATE ON repositories BEGIN
    UPDATE repositories_fts
    SET
        path = NEW.path,
        description = NEW.description,
        languages = IFNULL((
            SELECT GROUP_CONCAT(l.language_name, ' ' ORDER BY l.language_name)
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
        SELECT GROUP_CONCAT(l.language_name, ' ' ORDER BY l.language_name)
        FROM repository_languages rl
        JOIN languages l ON rl.language_id = l.id
        WHERE rl.repository_id = NEW.repository_id
    ), '')
    WHERE rowid = NEW.repository_id;
END;

CREATE TRIGGER repository_languages_delete AFTER DELETE ON repository_languages BEGIN
    UPDATE repositories_fts
    SET languages = IFNULL((
        SELECT GROUP_CONCAT(l.language_name, ' ' ORDER BY l.language_name)
        FROM repository_languages rl
        JOIN languages l ON rl.language_id = l.id
        WHERE rl.repository_id = OLD.repository_id
    ), '')
    WHERE rowid = OLD.repository_id;
END;