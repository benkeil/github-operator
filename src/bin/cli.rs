use clap::{Parser, Subcommand};
use tracing::event;

use github_operator::adapter::http_github_service::HttpGithubService;
use github_operator::domain::get_repository_use_case::GetRepositoryUseCase;
use github_operator::init_tracing;

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
    init_tracing().expect("Failed to initialize logging");

    let args = Cli::parse();

    let github_service = HttpGithubService::from_env();
    let get_github_repository_use_case = GetRepositoryUseCase::new(Box::new(github_service));

    match args.command {
        Commands::Get {
            repository: ref full_name,
        } => {
            event!(tracing::Level::INFO, "get {}", full_name);
            if let Ok(github_repository) = get_github_repository_use_case.execute(full_name).await {
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
            event!(tracing::Level::INFO, "set {}", repository);
        }
    }
}
