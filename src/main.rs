use clap::{Parser, Subcommand};
use db::{get_most_frequent_languages, search_repositories};
use gitlab::updater::GitLabUpdater;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::{env, error::Error};

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
        group: String,

        #[arg(long, requires = "group", help = "Get all projects in a group")]
        update: bool,
    },
    Stats,
    Search {
        #[arg(value_name = "search query")]
        query: String,

        #[arg(long, help = "Return result as json")]
        json: bool,

        #[arg(long, help = "Include archived repositories in search results")]
        include_archived: bool,
    },
}

#[derive(Serialize, Deserialize)]
struct DevSecConfig {
    test: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    match &cli.command {
        Commands::Gitlab {
            auth,
            group,
            update,
        } => {
            let gitlab_updater = GitLabUpdater::new(&auth, group, pool);
            if *update {
                match gitlab_updater.update().await {
                    Ok(_) => println!("GitLab data updated successfully."),
                    Err(e) => eprintln!("❌ Error updating GitLab data: {}", e),
                }
            }
        }
        Commands::Stats => {
            let most_used = get_most_frequent_languages(&pool).await?;
            most_used
                .iter()
                .for_each(|lang| println!("{}: {:.2}%", lang.0, lang.1));
        }
        Commands::Search {
            query,
            json,
            include_archived,
        } => {
            let result = search_repositories(&pool, &query, *include_archived).await?;

            if *json {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                for repo in result {
                    println!("{}", repo.web_url);
                }
            }
        }
    };

    Ok(())
}
