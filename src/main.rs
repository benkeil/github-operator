use kube::api::ListParams;
use kube::{Api, Client};
use tokio::task::JoinSet;
use tracing::event;

use github_operator::{init_tracing, ControllerError};

use crate::adapter::http_github_service::HttpGithubService;
use crate::controller::autolink_reference_controller::{self, AutolinkReferenceControllerContext};
use crate::controller::permission_controller;
use crate::controller::permission_controller::PermissionControllerContext;
use crate::controller::repository_controller::{self, RepositoryControllerContext};
use crate::domain::archive_repository_use_case::ArchiveRepositoryUseCase;
use crate::domain::delete_autolink_reference_use_case::DeleteAutolinkReferenceUseCase;
use crate::domain::delete_permissions_use_case::DeletePermissionUseCase;
use crate::domain::model::autolink_reference::AutolinkReference;
use crate::domain::model::permission::RepositoryPermission;
use crate::domain::model::repository::Repository;
use crate::domain::reconcile_autolink_reference_use_case::ReconcileAutolinkReferenceUseCase;
use crate::domain::reconcile_permissions_use_case::ReconcilePermissionUseCase;
use crate::domain::reconcile_repository_use_case::ReconcileRepositoryUseCase;

mod adapter;
mod controller;
mod domain;
mod extensions;

#[tokio::main]
async fn main() -> Result<(), ControllerError> {
    init_tracing()?;
    event!(tracing::Level::INFO, "starting controllers...");

    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;

    // the kubernetes API for our CRD
    let repository_api = Api::<Repository>::all(client.clone());
    let autolink_reference_api = Api::<AutolinkReference>::all(client.clone());
    let permission_api = Api::<RepositoryPermission>::all(client.clone());

    // check if the CRD is installed, or else throw an error
    repository_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;
    autolink_reference_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;
    permission_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;

    // dependencies
    let github_service = HttpGithubService::from_env();

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
            client: client.clone(),
            autolink_reference_api,
            reconcile_use_case: ReconcileAutolinkReferenceUseCase::new(Box::new(
                github_service.clone(),
            )),
            delete_use_case: DeleteAutolinkReferenceUseCase::new(Box::new(github_service.clone())),
        },
    ));

    // add permission controller
    tasks.spawn(permission_controller::run(PermissionControllerContext {
        client: client.clone(),
        permission_api,
        reconcile_use_case: ReconcilePermissionUseCase::new(Box::new(github_service.clone())),
        delete_use_case: DeletePermissionUseCase::new(Box::new(github_service.clone())),
    }));

    while let Some(res) = tasks.join_next().await {
        if let Err(e) = res {
            event!(tracing::Level::ERROR, "error: {:?}", e);
        }
    }

    Ok(())
}
