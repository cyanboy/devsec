CREATE TABLE vulnerabilities(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    severity TEXT NOT NULL CHECK ( severity IN ( 'LOW', 'MEDIUM', 'HIGH', 'CRITICAL' ) ),
    cve TEXT NOT NULL UNIQUE,
    cvss_score REAL NOT NULL CHECK ( cvss_score >= 0.0 AND cvss_score <= 10.0 )
);

CREATE TABLE repository_vulnerability (
    repository_id INTEGER NOT NULL,
    vulnerability_id INTEGER NOT NULL,
    vulnerability_status TEXT NOT NULL,
    status TEXT NOT NULL,
    PRIMARY KEY (repository_id, vulnerability_id),
    FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE,
    FOREIGN KEY (vulnerability_id) REFERENCES vulnerabilities (id) ON DELETE CASCADE
);
