use clap::{Parser, Subcommand};
use db::init_db;
use gitlab::service::GitLabUpdaterService;
use repositories::{get_most_frequent_languages, search_repositories};
use std::error::Error;

mod db;
mod gitlab;
mod progress_bar;
mod repositories;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Gitlab {
        #[command(subcommand)]
        action: GitlabCommands,
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

#[derive(Subcommand)]
enum GitlabCommands {
    Update {
        #[arg(long, value_name = "GITLAB_TOKEN")]
        auth: String,

        #[arg(short, long, value_name = "GitLab group id")]
        group_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let pool = init_db().await?;

    match &cli.command {
        Some(Commands::Gitlab { action }) => match action {
            GitlabCommands::Update { auth, group_id } => {
                let gitlab_updater = GitLabUpdaterService::new(auth, group_id, pool);

                match gitlab_updater.update().await {
                    Ok(_) => (),
                    Err(e) => eprintln!("❌ Error updating GitLab data: {}", e),
                }
            }
        },
        Some(Commands::Stats) => {
            let most_used = get_most_frequent_languages(&pool).await?;
            most_used
                .iter()
                .for_each(|lang| println!("{}: {:.2}%", lang.0, lang.1));
        }
        Some(Commands::Search {
            query,
            json,
            include_archived,
        }) => {
            let result = search_repositories(&pool, query, *include_archived).await?;

            if *json {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                for repo in result {
                    println!("{}", repo.web_url);
                }
            }
        }
        None => {
            todo!("Implement tui")
        }
    };

    Ok(())
}
