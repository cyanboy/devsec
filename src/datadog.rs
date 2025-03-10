use std::error::Error;

use crossterm::style::Stylize;
use serde::Deserialize;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{
    repositories::find_repository_by_path,
    vulnerabilities::{NewVulnerability, Severity},
};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DatadogVulnerability {
    #[serde(rename = "Severity")]
    severity: DatadogSeverity,

    #[serde(rename = "CVSS Base Score")]
    cvss_base_score: f64,

    #[serde(rename = "Vulnerability")]
    vulnerability: String,

    #[serde(rename = "Resource Type")]
    resource_type: ResourceType,

    #[serde(rename = "Resource Name")]
    resource_name: String,

    #[serde(rename = "Status")]
    status: Status,

    #[serde(rename = "CVE")]
    cve: String,

    #[serde(rename = "Library Name")]
    library_name: String,

    #[serde(rename = "Last Detected", with = "time::serde::rfc3339")]
    last_detected: OffsetDateTime,

    #[serde(rename = "First Detected", with = "time::serde::rfc3339")]
    first_detected: OffsetDateTime,

    #[serde(rename = "Registry")]
    registry: String,

    #[serde(rename = "Repository")]
    repository: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum DatadogSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Deserialize)]
pub enum ResourceType {
    #[serde(rename = "Container Image")]
    ContainerImage,
    Host,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Status {
    Open,
    AutoClosed,
}

pub async fn process_csv(
    pool: &SqlitePool,
    filename: &str,
    _group_id: &str,
) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(filename)?;

    for result in rdr.deserialize() {
        let dd_vuln: DatadogVulnerability = result?;

        let _new_vuln = NewVulnerability {
            name: dd_vuln.vulnerability,
            severity: match dd_vuln.severity {
                DatadogSeverity::Critical => Severity::Critical,
                DatadogSeverity::High => Severity::High,
                DatadogSeverity::Medium => Severity::Medium,
                DatadogSeverity::Low => Severity::Low,
            },
            cve: dd_vuln.cve,
            cvss_score: dd_vuln.cvss_base_score,
        };

        // let mut tx = pool.begin().await?;

        let _resource_name = dd_vuln.resource_name;
        let _registry = dd_vuln.registry;
        let repository = dd_vuln.repository;

        if let Some(name) = repository {
            let repo = find_repository_by_path(pool, &name).await;

            println!(
                "{} {}",
                name.green(),
                repo.map_or(String::new(), |x| x.path).blue()
            );
        }

        // let vuln = insert_vulnerability_and_verify(&mut tx, new_vuln).await?;

        // let repo_vuln = RepositoryVulnerability {
        //     repository_id: todo!(),
        //     vulnerability_id: vuln.id,
        //     status: todo!(),
        // };

        // insert_repository_vuln_and_verify(&mut tx, &repo_vuln).await?;

        // tx.commit().await?;
    }

    Ok(())
}
