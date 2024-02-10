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

use crate::controller::finalizer_name;
use crate::domain::delete_autolink_reference_use_case::DeleteAutolinkReferenceUseCase;
use crate::domain::model::autolink_reference::{AutolinkReference, AutolinkReferenceStatus};
use crate::domain::reconcile_autolink_reference_use_case::ReconcileAutolinkReferenceUseCase;
use crate::extensions::DurationExtension;
use crate::ControllerError;

pub async fn run(
    controller_context: AutolinkReferenceControllerContext,
) -> Result<(), ControllerError> {
    Controller::new(
        controller_context.autolink_reference_api.clone(),
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
    autolink_reference: Arc<AutolinkReference>,
    ctx: Arc<Context>,
) -> Result<Action, ControllerError> {
    log::info!("reconcile: {:?}", autolink_reference.object_ref(&()));
    // must be namespaced
    let recorder = Recorder::new(
        ctx.client.clone(),
        "autolink-reference-github-controller".into(),
        autolink_reference.object_ref(&()),
    );
    let autolink_reference_api = Api::<AutolinkReference>::namespaced(
        ctx.client.clone(),
        autolink_reference
            .metadata
            .namespace
            .as_ref()
            .ok_or_else(|| ControllerError::IllegalDocument)?,
    );

    finalizer(
        &autolink_reference_api,
        finalizer_name("autolink-reference").as_str(),
        autolink_reference,
        |event| async {
            match event {
                Event::Apply(autolink_reference) => {
                    log::info!("object ref: {:?}", autolink_reference.object_ref(&()));
                    match ctx
                        .reconcile_use_case
                        .execute(&autolink_reference, recorder)
                        .await
                    {
                        Ok(id) => {
                            update_status(
                                &autolink_reference_api,
                                autolink_reference,
                                Some(id),
                                None,
                            )
                            .await?;
                            Ok(Action::requeue(Duration::from_minutes(1)))
                        }
                        Err(e) => {
                            log::error!("reconcile failed: {:?}", e);
                            update_status(
                                &autolink_reference_api,
                                autolink_reference,
                                None,
                                Some(e),
                            )
                            .await?;
                            Ok(Action::requeue(Duration::from_secs(5)))
                        }
                    }
                }
                Event::Cleanup(github_repository) => {
                    match ctx
                        .delete_use_case
                        .execute(&github_repository, recorder)
                        .await
                    {
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
    _github_repository: Arc<AutolinkReference>,
    error: &ControllerError,
    _ctx: Arc<Context>,
) -> Action {
    log::warn!("reconcile failed: {:?}", error,);
    Action::requeue(Duration::from_secs(5))
}

async fn update_status(
    api: &Api<AutolinkReference>,
    autolink_reference: Arc<AutolinkReference>,
    id: Option<u32>,
    e: Option<ControllerError>,
) -> Result<(), ControllerError> {
    let name = autolink_reference.name_unchecked();
    let ready = match e {
        Some(_) => Condition {
            type_: "Ready".into(),
            status: "False".into(),
            reason: "ReconcileFailed".into(),
            message: "Reconcile failed".into(),
            last_transition_time: Time(chrono::Utc::now()),
            observed_generation: autolink_reference.metadata.generation,
        },
        None => Condition {
            type_: "Ready".into(),
            status: "True".into(),
            reason: "ReconcileSucceed".into(),
            message: "Reconcile succeed".into(),
            last_transition_time: Time(chrono::Utc::now()),
            observed_generation: autolink_reference.metadata.generation,
        },
    };
    let conditions = vec![ready];
    let healthy = match e {
        Some(_) => Some(false),
        None => Some(true),
    };
    let status = json!({
        "status": AutolinkReferenceStatus {
            conditions,
            healthy,
            id,
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

pub struct AutolinkReferenceControllerContext {
    /// Kubernetes client
    pub client: Client,
    pub autolink_reference_api: Api<AutolinkReference>,
    pub reconcile_use_case: ReconcileAutolinkReferenceUseCase,
    pub delete_use_case: DeleteAutolinkReferenceUseCase,
}

pub struct Context {
    /// Kubernetes client
    pub client: Client,
    pub autolink_reference_api: Api<AutolinkReference>,
    pub reconcile_use_case: ReconcileAutolinkReferenceUseCase,
    pub delete_use_case: DeleteAutolinkReferenceUseCase,
}

impl From<AutolinkReferenceControllerContext> for Arc<Context> {
    fn from(controller_context: AutolinkReferenceControllerContext) -> Self {
        Arc::new(Context {
            client: controller_context.client,
            autolink_reference_api: controller_context.autolink_reference_api,
            reconcile_use_case: controller_context.reconcile_use_case,
            delete_use_case: controller_context.delete_use_case,
        })
    }
}
