use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use kube::api::ListParams;
use kube::runtime::controller::Action;
use kube::runtime::finalizer::{finalizer, Event};
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::{Api, Client, Resource};

use github_operator::ControllerError;

use crate::adapter::octocrab_github_service::OctocrabGitHubService;
use crate::domain::model::github_repository::GitHubRepository;
use crate::domain::reconcile_github_repository_use_case::ReconcileGitHubRepositoryUseCase;
use crate::extensions::DurationExtension;

pub async fn run(state: State) -> Result<(), ControllerError> {
    let client = Client::try_default()
        .await
        .map_err(ControllerError::KubeError)?;
    let github_client = octocrab::instance();
    let github_service = OctocrabGitHubService::new(github_client);
    let use_case = ReconcileGitHubRepositoryUseCase::new(github_service);

    let github_repository_api = Api::<GitHubRepository>::all(client.clone());
    github_repository_api
        .list(&ListParams::default().limit(1))
        .await
        .map_err(ControllerError::CrdNotFound)?;

    Controller::new(
        github_repository_api.clone(),
        Config::default().any_semantic(),
    )
    .shutdown_on_signal()
    .run(
        |github_repository, _| {
            let namespace = github_repository.meta().namespace.as_ref().unwrap();
            let api = Api::<GitHubRepository>::namespaced(client.clone(), namespace);
            let use_case = use_case.clone();
            async move {
                finalizer(
                    &api,
                    "github-repository-controller.platform.benkeil.de/cleanup",
                    github_repository,
                    |event| async {
                        match event {
                            Event::Apply(github_repository) => {
                                let result: Result<Action, ControllerError> =
                                    match use_case.execute(github_repository).await {
                                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                                    };
                                result
                            }
                            Event::Cleanup(github_repository) => {
                                let result: Result<Action, ControllerError> =
                                    match use_case.execute(github_repository).await {
                                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                                    };
                                result
                            }
                        }
                    },
                )
                .await
            }
        },
        |_obj, _err, _| Action::requeue(Duration::from_secs(10)),
        Arc::new(()),
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

async fn apply(
    github_repository: Arc<GitHubRepository>,
    _github_repository_api: &Api<GitHubRepository>,
    use_case: &ReconcileGitHubRepositoryUseCase,
) -> Result<Action, ControllerError> {
    log::info!("apply");
    use_case.execute(github_repository).await;
    Ok(Action::requeue(Duration::from_secs(30)))
}

async fn cleanup(
    _github_repository: Arc<GitHubRepository>,
    _github_repository_api: &Api<GitHubRepository>,
) -> Result<Action, ControllerError> {
    log::info!("cleanup");
    Ok(Action::await_change())
}

async fn reconcile(
    github_repository: Arc<GitHubRepository>,
    ctx: Arc<Context>,
) -> Result<Action, ControllerError> {
    log::info!("reconcile {:?}", github_repository);
    match ctx.use_case.execute(github_repository).await {
        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
    }
}

fn error_policy(
    _github_repository: Arc<GitHubRepository>,
    error: &ControllerError,
    _ctx: Arc<Context>,
) -> Action {
    log::warn!("reconcile failed: {:?}", error);
    Action::requeue(Duration::from_secs(5))
}

#[derive(Clone, Debug)]
pub struct State {
    pub github_token: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            github_token: std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set"),
        }
    }
}

#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
    pub use_case: ReconcileGitHubRepositoryUseCase,
}
