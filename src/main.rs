use std::{env, error::Error};

use clap::{Parser, Subcommand};
use gitlab::updater::{gitlab_update_languages, gitlab_update_projects};
use sqlx::postgres::PgPoolOptions;

extern crate chrono;

mod codebase;
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
        #[arg(long, value_name = "GITLAB_TOKEN")]
        auth: String,

        #[arg(long, value_name = "Group id")]
        group_id: Option<String>,

        #[arg(long, requires = "group_id", help = "Get all projects in a group")]
        update_projects: bool,

        #[arg(long, help = "Get all projects in a group")]
        update_languages: bool,
    },
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
            update_projects,
            group_id,
            update_languages,
        } => {
            if *update_projects && group_id.is_some() {
                let _projects =
                    gitlab_update_projects(auth, group_id.clone().unwrap().as_str(), &pool)
                        .await
                        .map_err(|e| format!("Failed to fetch group projects: {}", e))?;
            }

            if *update_languages {
                let languages = gitlab_update_languages(auth, &pool).await?;
                println!("{:?}", languages);
            }
        }
    };

    Ok(())
}
