CREATE TABLE repositories (
    id SERIAL PRIMARY KEY,
    external_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    name TEXT NOT NULL,
    namespace TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    pushed_at TIMESTAMPTZ NOT NULL,
    ssh_url TEXT NOT NULL,
    web_url TEXT NOT NULL,
    private BOOLEAN NOT NULL,
    forks_count INTEGER NOT NULL,
    archived BOOLEAN NOT NULL,
    size REAL NOT NULL,
    UNIQUE (external_id, source)
);

CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    language_name TEXT UNIQUE NOT NULL
);

CREATE TABLE repository_languages (
    repository_id INTEGER NOT NULL REFERENCES repositories (id) ON DELETE CASCADE,
    language_id INTEGER NOT NULL REFERENCES languages (id) ON DELETE CASCADE,
    percentage REAL NOT NULL,
    PRIMARY KEY (repository_id, language_id)
);