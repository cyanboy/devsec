use sqlx::SqlitePool;

#[derive(Debug)]
pub struct NewVulnerability {
    pub name: String,
    pub severity: Severity,
    pub cve: String,
    pub cvss_score: f64,
}

#[derive(Debug)]
pub struct Vulnerability {
    pub id: i64,
    pub name: String,
    pub severity: Severity,
    pub cve: String,
    pub cvss_score: f64,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug)]
pub struct RepositoryVulnerability {
    pub repository_id: i64,
    pub vulnerability_id: i64,
    pub status: Status,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum Status {
    Open,
    Closed,
}

pub async fn insert_vulnerability(
    pool: &SqlitePool,
    v: NewVulnerability,
) -> Result<Vulnerability, sqlx::Error> {
    let vuln = sqlx::query_as!(
        Vulnerability,
        r#"
            INSERT INTO vulnerabilities (name, severity , cve, cvss_score)
            VALUES ( ?, ?, ?, ? )
            ON CONFLICT (cve) DO UPDATE
            SET
                name = name,
                severity = severity,
                cvss_score = cvss_score
            RETURNING
                id,
                name,
                severity as "severity: _",
                cve,
                cvss_score
            "#,
        v.name,
        v.severity,
        v.cve,
        v.cvss_score
    )
    .fetch_one(pool)
    .await?;

    Ok(vuln)
}

pub async fn insert_repository_vuln(
    pool: &SqlitePool,
    repo_vuln: &RepositoryVulnerability,
) -> Result<RepositoryVulnerability, sqlx::Error> {
    let _ = sqlx::query_as!(
        RepositoryVulnerability,
        r#"
        INSERT INTO repository_vulnerability (repository_id, vulnerability_id, status)
        VALUES ( ?, ?, ? )
        ON CONFLICT (repository_id, vulnerability_id)
        DO UPDATE SET status = status
        RETURNING
            repository_id,
            vulnerability_id,
            status as "status: _"
        "#,
        repo_vuln.repository_id,
        repo_vuln.vulnerability_id,
        repo_vuln.status,
    )
    .fetch_one(pool)
    .await?;

    todo!("implement")
}
