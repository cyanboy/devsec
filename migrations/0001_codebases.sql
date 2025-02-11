CREATE TABLE codebases (
    id SERIAL PRIMARY KEY,
    external_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    full_name TEXT NOT NULL,
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

CREATE TABLE codebase_languages (
    codebase_id INTEGER NOT NULL REFERENCES codebases (id) ON DELETE CASCADE, 
    language_id INTEGER NOT NULL REFERENCES languages (id) ON DELETE CASCADE,
    percentage REAL NOT NULL CHECK (
        percentage >= 0.00
        AND percentage <= 100.00
    ),
    PRIMARY KEY (codebase_id, language_id)
);