use crate::adapter::octocrab_github_service::OctocrabGitHubService;
use crate::controller::ControllerContext;
use crate::domain::reconcile_github_repository_use_case::ReconcileGitHubRepositoryUseCase;
use github_operator::ControllerError;
use kube::Client;

mod adapter;
mod controller;
mod domain;
mod extensions;

#[tokio::main]
async fn main() -> Result<(), ControllerError> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("start controller");

    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;
    let github_client = octocrab::instance();
    let github_service = OctocrabGitHubService::new(github_client);
    let use_case = ReconcileGitHubRepositoryUseCase::new(github_service);

    let state = ControllerContext {
        github_token: std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set"),
        client,
        use_case,
    };

    if let Err(e) = controller::run(state).await {
        log::error!("{}", e);
    }

    Ok(())
}
