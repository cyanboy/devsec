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
        group: String,

        #[arg(long, requires = "group", help = "Get all projects in a group")]
        update: bool,
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
            group,
            update,
        } => {
            let gitlab_updater = GitLabUpdater::new(&auth, group, pool);
            if *update {
                gitlab_updater.update().await?;
            }
        }
        Commands::Stats => {
            let most_used = get_most_frequent_languages(&pool).await?;
            most_used
                .iter()
                .for_each(|lang| println!("{}: {:.2}%", lang.0, lang.1));
        }
    };

    Ok(())
}
