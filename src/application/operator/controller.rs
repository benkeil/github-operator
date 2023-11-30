use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use k8s_openapi::api::core::v1::ObjectReference;
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::events::{Event as KubeEvent, EventType, Recorder, Reporter};
use kube::runtime::finalizer::{finalizer, Error, Event};
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::{Api, Client, Resource};
use serde_json::json;

use crate::domain::archive_github_repository_use_case::ArchiveGitHubRepositoryUseCase;
use crate::domain::model::github_repository;
use crate::domain::model::github_repository::{GitHubRepository, GitHubRepositoryStatus};
use crate::domain::model::repository::Repository;
use crate::domain::reconcile_github_repository_use_case::ReconcileGitHubRepositoryUseCase;
use crate::extensions::DurationExtension;
use crate::ControllerError;

pub async fn run(controller_context: ControllerContext) -> Result<(), ControllerError> {
    Controller::new(
        controller_context.github_repository_api.clone(),
        Config::default().any_semantic(),
    )
    .shutdown_on_signal()
    .run(reconcile, handle_errors, controller_context.into())
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
    // must be namespaced, and because of that here
    let recorder = Recorder::new(
        ctx.client.clone(),
        "github-repository-controller".into(),
        github_repository.object_ref(&()),
    );

    finalizer(
        &ctx.github_repository_api,
        "github-repository-controller.platform.benkeil.de/cleanup",
        github_repository,
        |event| async {
            match event {
                Event::Apply(github_repository) => {
                    log::info!("object ref: {:?}", github_repository.object_ref(&()));
                    match ctx
                        .reconcile_use_case
                        .execute(github_repository.clone(), recorder)
                        .await
                    {
                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    }
                }
                Event::Cleanup(github_repository) => {
                    match ctx.archive_use_case.execute(github_repository).await {
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

async fn update_status(
    repository: Repository,
    api: Api<GitHubRepository>,
) -> Result<(), ControllerError> {
    let url = "".to_string();
    let status = json!({
        "status": GitHubRepositoryStatus { }
    });
    api.patch_status(
        repository.full_name.as_str(),
        &PatchParams::default(),
        &Patch::Merge(status),
    )
    .await
    .map_err(|_| ControllerError::UseCaseError)?;
    Ok(())
}

pub struct ControllerContext {
    /// Kubernetes client
    pub client: Client,
    pub recorder: Recorder,
    pub github_repository_api: Api<GitHubRepository>,
    pub reconcile_use_case: ReconcileGitHubRepositoryUseCase,
    pub archive_use_case: ArchiveGitHubRepositoryUseCase,
}

pub struct Context {
    /// Kubernetes client
    pub client: Client,
    pub recorder: Recorder,
    pub github_repository_api: Api<GitHubRepository>,
    pub reconcile_use_case: ReconcileGitHubRepositoryUseCase,
    pub archive_use_case: ArchiveGitHubRepositoryUseCase,
}

impl From<ControllerContext> for Arc<Context> {
    fn from(controller_context: ControllerContext) -> Self {
        Arc::new(Context {
            client: controller_context.client,
            recorder: controller_context.recorder,
            github_repository_api: controller_context.github_repository_api,
            reconcile_use_case: controller_context.reconcile_use_case,
            archive_use_case: controller_context.archive_use_case,
        })
    }
}
