CREATE VIRTUAL
TABLE codebases_fts USING fts5 (
    path, description, languages,
    tokenize = "trigram"
);

INSERT INTO
    codebases_fts (
        rowid,
        path,
        description,
        languages
    )
SELECT cb.id, cb.path, cb.description, IFNULL(
        (
            SELECT GROUP_CONCAT(
                    l.name, ' '
                    ORDER BY l.name
                )
            FROM
                codebase_languages cl
                JOIN programming_languages l ON cl.language_id = l.id
            WHERE
                cl.codebase_id = cb.id
        ), ''
    )
FROM codebases cb;

CREATE TRIGGER codebases_fts_insert AFTER INSERT ON codebases BEGIN
    INSERT INTO codebases_fts (rowid, path, description, languages)
    VALUES (
        NEW.id,
        NEW.path,
        NEW.description,
        IFNULL((
            SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
            FROM codebase_languages cl
            JOIN programming_languages l ON cl.language_id = l.id
            WHERE cl.codebase_id = NEW.id
        ), '')
    );
END;

CREATE TRIGGER codebases_fts_update AFTER UPDATE ON codebases BEGIN
    UPDATE codebases_fts
    SET
        path = NEW.path,
        description = NEW.description,
        languages = IFNULL((
            SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
            FROM codebase_languages cl
            JOIN programming_languages l ON cl.language_id = l.id
            WHERE cl.codebase_id = NEW.id
        ), '')
    WHERE rowid = NEW.id;
END;

CREATE TRIGGER codebases_fts_delete AFTER DELETE ON codebases BEGIN
    DELETE FROM codebases_fts WHERE rowid = OLD.id;
END;

CREATE TRIGGER codebase_languages_insert AFTER INSERT ON codebase_languages BEGIN
    UPDATE codebases_fts
    SET languages = IFNULL((
        SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
        FROM codebase_languages cl
        JOIN programming_languages l ON cl.language_id = l.id
        WHERE cl.codebase_id = NEW.codebase_id
    ), '')
    WHERE rowid = NEW.codebase_id;
END;

CREATE TRIGGER codebase_languages_delete AFTER DELETE ON codebase_languages BEGIN
    UPDATE codebases_fts
    SET languages = IFNULL((
        SELECT GROUP_CONCAT(l.name, ' ' ORDER BY l.name)
        FROM codebase_languages cl
        JOIN programming_languages l ON cl.language_id = l.id
        WHERE cl.codebase_id = OLD.codebase_id
    ), '')
    WHERE rowid = OLD.codebase_id;
END;