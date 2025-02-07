use clap::{Parser, Subcommand};
use db::get_most_frequent_languages;
use gitlab::updater::GitLabUpdater;
use sqlx::postgres::PgPoolOptions;
use std::{env, error::Error};

extern crate chrono;

mod db;
mod gitlab;
mod progress_bar;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Gitlab {
        #[arg(long, value_name = "GITLAB_TOKEN", env = "GITLAB_TOKEN")]
        auth: String,

        #[arg(long, value_name = "GitLab group id", env = "GITLAB_GROUP_ID")]
        group_id: Option<String>,

        #[arg(long, requires = "group_id", help = "Get all projects in a group")]
        update_projects: bool,

        #[arg(long, help = "Update languages for GitLab projects")]
        update_languages: bool,
    },
    Stats,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    match &cli.command {
        Commands::Gitlab {
            auth,
            group_id,
            update_projects,
            update_languages,
        } => {
            let gitlab_updater = GitLabUpdater::new(&auth, group_id.clone(), pool);
            if *update_projects {
                gitlab_updater
                    .gitlab_update_projects()
                    .await
                    .map_err(|e| format!("Failed to fetch group projects: {}", e))?
            }
            if *update_languages {
                gitlab_updater.gitlab_update_languages().await?
            }
        }
        Commands::Stats => {
            let most_used = get_most_frequent_languages(&pool).await?;
            most_used.iter().for_each(|lang| println!("{:?}", lang));
        }
    };

    Ok(())
}
