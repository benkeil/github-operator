use std::sync::Arc;
use std::time::Duration;

use crate::controller::finalizer_name;
use futures::StreamExt;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{Condition, Time};
use k8s_openapi::chrono;
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::runtime::events::Recorder;
use kube::runtime::finalizer::{finalizer, Event};
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::{Api, Client, Resource, ResourceExt};
use serde_json::json;
use tracing::{instrument, Instrument};

use crate::domain::archive_repository_use_case::ArchiveRepositoryUseCase;
use crate::domain::model::repository::{Repository, RepositoryStatus};
use crate::domain::reconcile_repository_use_case::ReconcileRepositoryUseCase;
use crate::extensions::DurationExtension;
use crate::ControllerError;

pub async fn run(controller_context: RepositoryControllerContext) -> Result<(), ControllerError> {
    Controller::new(
        controller_context.repository_api.clone(),
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

#[instrument(ret, err, skip(object, ctx))]
async fn reconcile(
    object: Arc<Repository>,
    ctx: Arc<RepositoryControllerContext>,
) -> Result<Action, ControllerError> {
    log::info!("reconcile: {:?}", object.object_ref(&()));
    // must be namespaced
    let recorder = Recorder::new(
        ctx.client.clone(),
        "repository-github-controller".into(),
        object.object_ref(&()),
    );
    let github_repository_api = Api::<Repository>::namespaced(
        ctx.client.clone(),
        object
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| ControllerError::IllegalDocument)?,
    );

    finalizer(
        &github_repository_api,
        finalizer_name("repository").as_str(),
        object,
        |event| async {
            match event {
                Event::Apply(github_repository) => {
                    log::info!("object ref: {:?}", github_repository.object_ref(&()));
                    match ctx
                        .reconcile_use_case
                        .execute(&github_repository.spec, recorder)
                        .instrument(tracing::info_span!("apply"))
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
                    match ctx
                        .archive_use_case
                        .execute(&github_repository.spec)
                        .instrument(tracing::info_span!("cleanup"))
                        .await
                    {
                        Ok(_) => Ok(Action::requeue(Duration::from_minutes(1))),
                        Err(_) => Ok(Action::requeue(Duration::from_secs(5))),
                    }
                }
            }
        },
    )
    .instrument(tracing::info_span!("finalizer"))
    .await
    .map_err(|e| ControllerError::FinalizerError(Box::new(e)))
}

fn handle_errors(
    _github_repository: Arc<Repository>,
    error: &ControllerError,
    _ctx: Arc<RepositoryControllerContext>,
) -> Action {
    log::warn!("reconcile failed: {:?}", error,);
    Action::requeue(Duration::from_secs(5))
}

async fn update_status(
    github_repository: Arc<Repository>,
    api: &Api<Repository>,
    e: Option<ControllerError>,
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
        "status": RepositoryStatus {
            conditions,
            healthy,
        }
    });
    log::debug!("patching {} status with: {:#?}", name, status);
    api.patch_status(
        name.as_str(),
        &PatchParams::default(),
        &Patch::Merge(&status),
    )
    .await
    .map_err(ControllerError::KubeError)?;
    Ok(())
}

pub struct RepositoryControllerContext {
    /// Kubernetes client
    pub client: Client,
    pub repository_api: Api<Repository>,
    pub reconcile_use_case: ReconcileRepositoryUseCase,
    pub archive_use_case: ArchiveRepositoryUseCase,
}
