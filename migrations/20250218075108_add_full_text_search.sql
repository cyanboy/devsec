ALTER TABLE repositories ADD COLUMN search_vector tsvector;

UPDATE repositories
SET
    search_vector = to_tsvector (
        'english',
        coalesce(repo_name, '') || ' ' || coalesce(description, '')
    );

CREATE INDEX idx_repositories_search ON repositories USING GIN (search_vector);

CREATE FUNCTION update_search_vector() RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := to_tsvector('english', coalesce(NEW.repo_name, '') || ' ' || coalesce(NEW.description, ''));
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER repositories_search_vector_trigger
BEFORE INSERT OR UPDATE ON repositories
FOR EACH ROW EXECUTE FUNCTION update_search_vector();