CREATE TABLE codebases (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    full_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    pushed_at TIMESTAMPTZ NOT NULL,
    ssh_url TEXT NOT NULL,
    web_url TEXT NOT NULL,
    private BOOLEAN NOT NULL,
    forks_count INTEGER NOT NULL,
    default_branch TEXT NOT NULL,
    archived BOOLEAN NOT NULL
);

CREATE TABLE languages (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE codebase_languages (
    codebase_id INTEGER NOT NULL REFERENCES codebases(id) ON DELETE CASCADE,
    language_id INTEGER NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    percentage REAL NOT NULL CHECK (percentage >= 0.00 AND percentage <= 100.00),
    PRIMARY KEY (codebase_id, language_id)
);