use clap::{Parser, Subcommand};

use github_operator::adapter::octocrab_github_service::OctocrabGitHubService;
use github_operator::domain::port::github_service::GitHubService;
use github_operator::extensions::OctocrabExtensoin;

/// CLI to manage GitHub repositories
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "repository")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// show repository settings
    #[command(arg_required_else_help = true)]
    Get {
        /// The repository
        repository: String,
    },
    /// update repository settings
    #[command(arg_required_else_help = true)]
    Set {
        /// The repository
        repository: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let github_client = octocrab::OctocrabBuilder::from_env();
    let github_service = OctocrabGitHubService::new(github_client);

    match args.command {
        Commands::Get { repository } => {
            let (owner, name) = repository.split_once('/').unwrap();
            println!("get {}/{}", owner, name);
            let github_repository = github_service.get_repository(owner, name).await.unwrap();
            println!("{:#?}", github_repository);
        }
        Commands::Set { repository } => {
            println!("set {}", repository);
        }
    }
}
