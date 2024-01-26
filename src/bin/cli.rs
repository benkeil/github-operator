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
    #[arg(short, long)]
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, clap::ValueEnum, Clone)]
enum OutputFormat {
    Default,
    Json,
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
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let args = Cli::parse();

    let github_client = octocrab::OctocrabBuilder::from_env();
    let github_service = OctocrabGitHubService::new(github_client);

    match args.command {
        Commands::Get { repository } => {
            let (owner, name) = repository.split_once('/').unwrap();
            println!("get {}/{}", owner, name);
            if let Ok(github_repository) = github_service.get_repository(owner, name).await {
                match args.output_format {
                    Some(OutputFormat::Json) => {
                        let json = serde_json::to_string_pretty(&github_repository).unwrap();
                        println!("{}", json);
                    }
                    _ => {
                        println!("{:#?}", github_repository);
                    }
                }
            }
        }
        Commands::Set { repository } => {
            println!("set {}", repository);
        }
    }
}
