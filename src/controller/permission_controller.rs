use std::sync::Arc;
use std::time::Duration;

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
use tracing::Instrument;

use crate::controller::finalizer_name;
use crate::domain::delete_permissions_use_case::DeletePermissionUseCase;
use crate::domain::model::permission::{RepositoryPermission, RepositoryPermissionStatus};
use crate::domain::reconcile_permissions_use_case::ReconcilePermissionUseCase;
use crate::extensions::DurationExtension;
use crate::ControllerError;

pub async fn run(controller_context: PermissionControllerContext) -> Result<(), ControllerError> {
    Controller::new(
        controller_context.permission_api.clone(),
        Config::default().any_semantic(),
    )
    .shutdown_on_signal()
    .run(reconcile, handle_errors, controller_context.into())
    .for_each(|res| async move {
        match res {
            Ok(o) => log::info!("reconciled {:?}", o),
            Err(e) => log::warn!("reconcile failed: {:#?}", e),
        }
    })
    .await;

    Ok(())
}

async fn reconcile(
    custom_resource: Arc<RepositoryPermission>,
    ctx: Arc<PermissionControllerContext>,
) -> Result<Action, ControllerError> {
    log::info!("reconcile: {:?}", custom_resource.object_ref(&()));
    // must be namespaced
    let recorder = Recorder::new(
        ctx.client.clone(),
        "permission-github-controller".into(),
        custom_resource.object_ref(&()),
    );
    let api = Api::<RepositoryPermission>::namespaced(
        ctx.client.clone(),
        custom_resource
            .namespace()
            .as_ref()
            .ok_or_else(|| ControllerError::IllegalDocument)?,
    );

    finalizer(
        &api,
        finalizer_name("permission").as_str(),
        custom_resource,
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
                            update_status(github_repository, &api, None).await?;
                            Ok(Action::requeue(Duration::from_minutes(1)))
                        }
                        Err(e) => {
                            log::error!("reconcile failed: {:?}", e);
                            update_status(github_repository, &api, Some(e)).await?;
                            Ok(Action::requeue(Duration::from_secs(5)))
                        }
                    }
                }
                Event::Cleanup(permission) => {
                    match ctx
                        .delete_use_case
                        .execute(&permission.spec, recorder)
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
    _custom_resource: Arc<RepositoryPermission>,
    error: &ControllerError,
    _ctx: Arc<PermissionControllerContext>,
) -> Action {
    log::warn!("reconcile failed: {:?}", error,);
    Action::requeue(Duration::from_secs(5))
}

async fn update_status(
    custom_resource: Arc<RepositoryPermission>,
    api: &Api<RepositoryPermission>,
    e: Option<ControllerError>,
) -> Result<(), ControllerError> {
    let name = custom_resource.name_unchecked();
    let ready = match e {
        Some(_) => Condition {
            type_: "Ready".into(),
            status: "False".into(),
            reason: "ReconcileFailed".into(),
            message: "Reconcile failed".into(),
            last_transition_time: Time(chrono::Utc::now()),
            observed_generation: custom_resource.metadata.generation,
        },
        None => Condition {
            type_: "Ready".into(),
            status: "True".into(),
            reason: "ReconcileSucceed".into(),
            message: "Reconcile succeed".into(),
            last_transition_time: Time(chrono::Utc::now()),
            observed_generation: custom_resource.metadata.generation,
        },
    };
    let conditions = vec![ready];
    let healthy = match e {
        Some(_) => Some(false),
        None => Some(true),
    };
    let status = json!({
        "status": RepositoryPermissionStatus {
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

pub struct PermissionControllerContext {
    /// Kubernetes client
    pub client: Client,
    pub permission_api: Api<RepositoryPermission>,
    pub reconcile_use_case: ReconcilePermissionUseCase,
    pub delete_use_case: DeletePermissionUseCase,
}
