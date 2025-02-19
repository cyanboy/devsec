PRAGMA foreign_keys = ON;

CREATE TABLE repositories (
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
    private BOOLEAN NOT NULL CHECK (private IN (0, 1)),
    forks_count INTEGER NOT NULL,
    archived BOOLEAN NOT NULL CHECK (archived IN (0, 1)),
    size INTEGER NOT NULL,
    commit_count INTEGER NOT NULL,
    UNIQUE (external_id, source)
);

-- Languages Table
CREATE TABLE languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    language_name TEXT UNIQUE NOT NULL
);

CREATE TABLE repository_languages (
    repository_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    percentage REAL NOT NULL,
    PRIMARY KEY (repository_id, language_id),
    FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE
);