use kube::api::ListParams;
use kube::{Api, Client};
use tokio::task::JoinSet;

use github_operator::ControllerError;

use crate::adapter::octocrab_github_service::OctocrabGitHubService;
use crate::controller::autolink_reference_controller::{self, AutolinkReferenceControllerContext};
use crate::controller::repository_controller::{self, RepositoryControllerContext};
use crate::domain::archive_repository_use_case::ArchiveRepositoryUseCase;
use crate::domain::delete_autolink_reference_use_case::DeleteAutolinkReferenceUseCase;
use crate::domain::reconcile_autolink_reference_use_case::ReconcileAutolinkReferenceUseCase;
use crate::domain::reconcile_repository_use_case::ReconcileRepositoryUseCase;
use crate::domain::spec::autolink_reference_spec::AutolinkReference;
use crate::domain::spec::repository_spec::Repository;
use crate::extensions::OctocrabExtensoin;

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

    // the kubernetes API for our CRD
    let repository_api = Api::<Repository>::all(client.clone());
    let autolink_reference_api = Api::<AutolinkReference>::all(client.clone());

    // check if the CRD is installed, or else throw an error
    repository_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;
    autolink_reference_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;

    let github_client = octocrab::OctocrabBuilder::from_env();
    let github_service = OctocrabGitHubService::new(github_client);

    let mut tasks = JoinSet::new();

    // add repository controller
    tasks.spawn(repository_controller::run(RepositoryControllerContext {
        client: client.clone(),
        repository_api,
        reconcile_use_case: ReconcileRepositoryUseCase::new(Box::new(github_service.clone())),
        archive_use_case: ArchiveRepositoryUseCase::new(Box::new(github_service.clone())),
    }));

    // add autolink reference controller
    tasks.spawn(autolink_reference_controller::run(
        AutolinkReferenceControllerContext {
            client,
            autolink_reference_api,
            reconcile_use_case: ReconcileAutolinkReferenceUseCase::new(Box::new(
                github_service.clone(),
            )),
            delete_use_case: DeleteAutolinkReferenceUseCase::new(Box::new(github_service.clone())),
        },
    ));

    while let Some(res) = tasks.join_next().await {
        if let Err(e) = res {
            log::error!("error: {:?}", e);
        }
    }

    Ok(())
}
