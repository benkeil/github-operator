use differ_from_spec::DifferFromSpec;
use k8s_openapi::api::core::v1::ObjectReference;
use kube::runtime::events::{Event, EventType, Recorder};
use kube::Resource;

use crate::domain::model::autolink_reference::{
    AutolinkReference, AutolinkReferenceRequest, AutolinkReferenceStatus,
};
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

pub struct ReconcileAutolinkReferenceUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ReconcileAutolinkReferenceUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        autolink_reference: &AutolinkReference,
        recorder: Recorder,
    ) -> Result<u32, ControllerError> {
        log::info!("reconcile: {}", &autolink_reference.spec.full_name);

        let spec: AutolinkReferenceRequest = autolink_reference.spec.clone().into();

        // if we have an id, we need to update the autolink reference
        if let Some(AutolinkReferenceStatus { id: Some(id), .. }) = &autolink_reference.status {
            let result = self
                .github_service
                .get_autolink_reference(&autolink_reference.spec.full_name, id)
                .await;

            return match result {
                // if we have an id and the autolink reference exists, we need to update it
                Ok(actual_autolink_reference) => {
                    let actual_autolink_reference: AutolinkReferenceRequest =
                        actual_autolink_reference.into();
                    // if the spec differs from the actual autolink reference, we need to update it
                    if spec.differ_from_spec(&actual_autolink_reference) {
                        self.github_service
                            .delete_autolink_references(&autolink_reference.spec.full_name, id)
                            .await?;
                        let response = self
                            .github_service
                            .add_autolink_reference(&autolink_reference.spec.full_name, &spec)
                            .await?;
                        let reference = autolink_reference.object_ref(&());
                        self.publish_updated_event(recorder, &reference).await?;
                        Ok(response.id)
                    } else {
                        Ok(*id)
                    }
                }
                Err(ControllerError::NotFound) => {
                    // if we have an id and the autolink reference does not exist, we need to create it
                    let response = self
                        .github_service
                        .add_autolink_reference(&autolink_reference.spec.full_name, &spec)
                        .await?;
                    let reference = autolink_reference.object_ref(&());
                    self.publish_created_event(recorder, &reference).await?;
                    Ok(response.id)
                }
                Err(e) => Err(e),
            };
        }

        // if we don't have an id, we need to create the autolink reference
        let autolink_references = self
            .github_service
            .get_autolink_references(&autolink_reference.spec.full_name)
            .await?;

        // the autolink reference already exists
        if let Some(existing) = autolink_references
            .iter()
            .find(|&v| v.key_prefix == spec.key_prefix)
        {
            return Ok(existing.id);
        }

        // the autolink reference does not exist
        let response = self
            .github_service
            .add_autolink_reference(&autolink_reference.spec.full_name, &spec)
            .await?;
        let reference = autolink_reference.object_ref(&());
        self.publish_created_event(recorder, &reference).await?;

        Ok(response.id)
    }

    async fn publish_created_event(
        &self,
        recorder: Recorder,
        reference: &ObjectReference,
    ) -> Result<(), ControllerError> {
        recorder
            .publish(
                &Event {
                    action: "autolink-reference-created".into(),
                    reason: "Reconciling".into(),
                    note: Some("Autolink reference created".into()),
                    type_: EventType::Normal,
                    secondary: None,
                },
                reference,
            )
            .await
            .map_err(ControllerError::KubeError)
    }

    async fn publish_updated_event(
        &self,
        recorder: Recorder,
        reference: &ObjectReference,
    ) -> Result<(), ControllerError> {
        recorder
            .publish(
                &Event {
                    action: "autolink-reference-updated".into(),
                    reason: "Reconciling".into(),
                    note: Some("Autolink reference updated".into()),
                    type_: EventType::Normal,
                    secondary: None,
                },
                reference,
            )
            .await
            .map_err(ControllerError::KubeError)
    }
}
