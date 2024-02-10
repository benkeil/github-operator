use differ_from_spec::DifferFromSpec;
use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::autolink_reference::AutolinkReferenceRequest;
use crate::domain::service::github_service::GitHubService;
use crate::domain::spec::autolink_reference_spec::{AutolinkReference, AutolinkReferenceStatus};
use crate::ControllerError;

pub struct ReconcileAutolinkReferenceUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ReconcileAutolinkReferenceUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    /// possible cases
    ///
    /// | spec | id   | exists | differ | action |
    /// |------|------|--------|--------|--------|
    /// | None | -    | -      | -      | None   |
    /// | Some | -    | false  | -      | Create |
    /// | Some | None | true   | false  | Import |
    /// | Some | -    | true   | true   | Delete + Create |
    /// | Some | Some | true   | false  | None   |
    ///
    pub async fn execute(
        &self,
        autolink_reference: &AutolinkReference,
        recorder: Recorder,
    ) -> Result<u32, ControllerError> {
        log::info!("reconcile: {}", &autolink_reference.spec.full_name);

        let autolink_reference_spec: AutolinkReferenceRequest =
            autolink_reference.spec.clone().into();

        let id = if let Some(AutolinkReferenceStatus { id: Some(id), .. }) =
            &autolink_reference.status
        {
            // throws an error if not found, which lead the status to be updated (remove the id) and trigger a retry
            let actual_autolink_reference: AutolinkReferenceRequest = self
                .github_service
                .get_autolink_reference(&autolink_reference.spec.full_name, id)
                .await?
                .into();

            if autolink_reference_spec.differ_from_spec(&actual_autolink_reference) {
                self.github_service
                    .delete_autolink_references(&autolink_reference.spec.full_name, id)
                    .await?;
                let response = self
                    .github_service
                    .add_autolink_reference(
                        &autolink_reference.spec.full_name,
                        &autolink_reference_spec,
                    )
                    .await?;
                recorder
                    .publish(Event {
                        action: "autolink-reference-updated".into(),
                        reason: "Reconciling".into(),
                        note: Some("Autolink reference updated".into()),
                        type_: EventType::Normal,
                        secondary: None,
                    })
                    .await
                    .map_err(ControllerError::KubeError)?;
                response.id
            } else {
                *id
            }
        } else {
            let autolink_references = self
                .github_service
                .get_autolink_references(&autolink_reference.spec.full_name)
                .await?;

            if let Some(existing) = autolink_references
                .iter()
                .find(|&r| r.key_prefix == autolink_reference_spec.key_prefix)
            {
                return Ok(existing.id);
            }

            let response = self
                .github_service
                .add_autolink_reference(
                    &autolink_reference.spec.full_name,
                    &autolink_reference_spec,
                )
                .await?;
            recorder
                .publish(Event {
                    action: "autolink-reference-created".into(),
                    reason: "Reconciling".into(),
                    note: Some("Autolink reference created".into()),
                    type_: EventType::Normal,
                    secondary: None,
                })
                .await
                .map_err(ControllerError::KubeError)?;
            response.id
        };

        Ok(id)
    }
}
