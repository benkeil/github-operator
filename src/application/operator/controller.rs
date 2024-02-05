use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use k8s_openapi::chrono;
use kube::api::{Patch, PatchParams, PostParams};
use kube::runtime::controller::Action;
use kube::runtime::events::Recorder;
use kube::runtime::finalizer::{finalizer, Event};
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::{Api, Client, Resource, ResourceExt};
use serde_json::json;

use crate::domain::archive_github_repository_use_case::ArchiveGitHubRepositoryUseCase;
use crate::domain::model::github_repository_spec::{GitHubRepository, GitHubRepositoryStatus};
use crate::domain::reconcile_github_repository_use_case::{
    ReconcileGitHubRepositoryUseCase, ReconcileGitHubRepositoryUseCaseError,
};
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
) -> Result<Action, ControllerError> {
    log::info!("reconcile: {:?}", github_repository.object_ref(&()));
    // must be namespaced
    let recorder = Recorder::new(
        ctx.client.clone(),
        "github-repository-controller".into(),
        github_repository.object_ref(&()),
    );
    let github_repository_api = Api::<GitHubRepository>::namespaced(
        ctx.client.clone(),
        github_repository
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| ControllerError::IllegalDocument)?,
    );

    finalizer(
        &github_repository_api,
        "github-repository-controller.platform.benkeil.de/cleanup",
        github_repository,
        |event| async {
            match event {
                Event::Apply(github_repository) => {
                    log::info!("object ref: {:?}", github_repository.object_ref(&()));
                    match ctx
                        .reconcile_use_case
                        .execute(&github_repository.spec, recorder)
                        .await
                    {
                        Ok(_) => {
                            update_status(github_repository, &github_repository_api, None).await?;
                            Ok(Action::requeue(Duration::from_minutes(1)))
                        }
                        Err(e) => {
                            log::error!("reconcile failed: {:?}", e);
                            update_status(github_repository, &github_repository_api, Some(e))
                                .await?;
                            Ok(Action::requeue(Duration::from_secs(5)))
                        }
                    }
                }
                Event::Cleanup(github_repository) => {
                    match ctx.archive_use_case.execute(&github_repository.spec).await {
                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    }
                }
            }
        },
    )
    .await
    .map_err(|e| ControllerError::FinalizerError(Box::new(e)))
}

fn handle_errors(
    _github_repository: Arc<GitHubRepository>,
    error: &ControllerError,
    _ctx: Arc<Context>,
) -> Action {
    log::warn!("reconcile failed: {:?}", error,);
    Action::requeue(Duration::from_secs(5))
}

async fn update_status(
    github_repository: Arc<GitHubRepository>,
    api: &Api<GitHubRepository>,
    e: Option<ReconcileGitHubRepositoryUseCaseError>,
) -> Result<(), ControllerError> {
    let name = github_repository.name_unchecked();
    let ready = match e {
        Some(_) => Condition {
            type_: "Ready".into(),
            status: "False".into(),
            reason: "ReconcileFailed".into(),
            message: "Reconcile failed".into(),
            last_transition_time: Time(chrono::Utc::now()),
            observed_generation: github_repository.metadata.generation,
        },
        None => Condition {
            type_: "Ready".into(),
            status: "True".into(),
            reason: "ReconcileSucceed".into(),
            message: "Reconcile succeed".into(),
            last_transition_time: Time(chrono::Utc::now()),
            observed_generation: github_repository.metadata.generation,
        },
    };
    let conditions = vec![ready];
    let healthy = match e {
        Some(_) => Some(false),
        None => Some(true),
    };
    let status = json!({
        "status": GitHubRepositoryStatus {
            conditions,
            healthy,
        }
    });
    log::debug!("patching {} status with: {:#?}", name, status);
    let a = api
        .patch_status(
            name.as_str(),
            &PatchParams::default(),
            &Patch::Merge(&status),
        )
        .await
        .map_err(ControllerError::KubeError)?;
    log::debug!("patched status: {:#?}", a);
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
