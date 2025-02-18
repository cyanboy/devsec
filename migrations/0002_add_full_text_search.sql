ALTER TABLE repositories ADD COLUMN search_vector tsvector;

UPDATE repositories
SET
    search_vector = setweight (
        to_tsvector ('english', coalesce(name, '')),
        'A'
    ) || setweight (
        to_tsvector (
            'english',
            coalesce(namespace, '')
        ),
        'B'
    ) || setweight (
        to_tsvector (
            'english',
            coalesce(description, '')
        ),
        'C'
    ) || setweight (
        to_tsvector (
            'english',
            coalesce(
                (
                    SELECT string_agg (language_name, ' ')
                    FROM
                        repository_languages
                        JOIN languages ON repository_languages.language_id = languages.id
                    WHERE
                        repository_languages.repository_id = repositories.id
                ),
                ''
            )
        ),
        'D'
    );

CREATE INDEX idx_repositories_search ON repositories USING GIN (search_vector);

CREATE FUNCTION update_search_vector() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', coalesce(NEW.name, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(NEW.namespace, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(NEW.description, '')), 'C') ||
        setweight(to_tsvector('english', coalesce(
            (SELECT string_agg(language_name, ' ')
             FROM repository_languages
             JOIN languages ON repository_languages.language_id = languages.id
             WHERE repository_languages.repository_id = NEW.id), ''
        )), 'D');
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER repositories_search_vector_trigger
BEFORE INSERT OR UPDATE ON repositories
FOR EACH ROW EXECUTE FUNCTION update_search_vector();


CREATE FUNCTION update_repository_search_on_language_change() RETURNS TRIGGER AS $$
BEGIN
    UPDATE repositories
    SET search_vector = search_vector
    WHERE id = NEW.repository_id;
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_repository_search_on_language_insert
AFTER INSERT OR DELETE ON repository_languages
FOR EACH ROW EXECUTE FUNCTION update_repository_search_on_language_change();