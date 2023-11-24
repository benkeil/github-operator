use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use kube::api::ListParams;
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Error, Event};
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::{Api, Client, Resource};

use github_operator::ControllerError;

use crate::domain::model::github_repository::GitHubRepository;
use crate::domain::reconcile_github_repository_use_case::ReconcileGitHubRepositoryUseCase;
use crate::extensions::DurationExtension;

pub async fn run(controller_context: ControllerContext) -> Result<(), ControllerError> {
    // the kubernetes API for our CRD
    let github_repository_api = Api::<GitHubRepository>::all(controller_context.client.clone());

    // check if the CRD is installed, or else throw an error
    github_repository_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;

    Controller::new(github_repository_api, Config::default().any_semantic())
        .shutdown_on_signal()
        .run(
            reconcile,
            handle_errors,
            Arc::new(Context {
                client: controller_context.client,
                use_case: controller_context.use_case,
            }),
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => log::info!("reconciled {:?}", o),
                Err(e) => log::warn!("reconcile failed: {}", e),
            }
        })
        .await;

    Ok(())
}

async fn reconcile(
    github_repository: Arc<GitHubRepository>,
    ctx: Arc<Context>,
) -> Result<Action, Error<ControllerError>> {
    log::info!("reconcile {:?}", github_repository);
    let namespace = github_repository.meta().namespace.as_ref().unwrap();
    let client = ctx.client.clone();
    let api = Api::<GitHubRepository>::namespaced(client, namespace);
    finalizer(
        &api,
        "github-repository-controller.platform.benkeil.de/cleanup",
        github_repository,
        |event| async {
            match event {
                Event::Apply(github_repository) => {
                    match ctx.use_case.execute(github_repository).await {
                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    }
                }
                Event::Cleanup(github_repository) => {
                    match ctx.use_case.execute(github_repository).await {
                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    }
                }
            }
        },
    )
    .await
}

fn handle_errors(
    _github_repository: Arc<GitHubRepository>,
    error: &Error<ControllerError>,
    _ctx: Arc<Context>,
) -> Action {
    log::warn!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5))
}

pub struct ControllerContext {
    pub github_token: String,
    /// Kubernetes client
    pub client: Client,
    pub use_case: ReconcileGitHubRepositoryUseCase,
}

pub struct Context {
    /// Kubernetes client
    pub client: Client,
    pub use_case: ReconcileGitHubRepositoryUseCase,
}
