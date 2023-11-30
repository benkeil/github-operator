use k8s_openapi::api::core::v1::ObjectReference;
use kube::api::ListParams;
use kube::runtime::events::Recorder;
use kube::{Api, Client};

use github_operator::ControllerError;

use crate::adapter::octocrab_github_service::OctocrabGitHubService;
use crate::application::operator::controller::{self, ControllerContext};
use crate::domain::archive_github_repository_use_case::ArchiveGitHubRepositoryUseCase;
use crate::domain::model::github_repository::GitHubRepository;
use crate::domain::reconcile_github_repository_use_case::ReconcileGitHubRepositoryUseCase;
use crate::extensions::OctocrabExtensoin;

mod adapter;
mod application;
mod domain;
mod extensions;

#[tokio::main]
async fn main() -> Result<(), ControllerError> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("start controller");

    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;
    // the kubernetes API for our CRD
    let github_repository_api = Api::<GitHubRepository>::all(client.clone());

    // check if the CRD is installed, or else throw an error
    github_repository_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;

    let recorder = Recorder::new(
        client.clone(),
        "github-repository-controller".into(),
        ObjectReference {
            ..Default::default()
        },
    );

    let github_client = octocrab::OctocrabBuilder::from_env();
    let github_service = OctocrabGitHubService::new(github_client);
    let reconcile_use_case =
        ReconcileGitHubRepositoryUseCase::new(Box::new(github_service.clone()));
    let archive_use_case = ArchiveGitHubRepositoryUseCase::new(Box::new(github_service));
    let state = ControllerContext {
        client,
        recorder,
        github_repository_api,
        reconcile_use_case,
        archive_use_case,
    };

    if let Err(e) = controller::run(state).await {
        log::error!("{}", e);
    }

    Ok(())
}
