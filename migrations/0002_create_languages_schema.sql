CREATE TABLE languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE repository_languages (
    repository_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    percentage REAL NOT NULL,
    PRIMARY KEY (repository_id, language_id),
    FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
    FOREIGN KEY (language_id) REFERENCES languages (id) ON DELETE CASCADE
);