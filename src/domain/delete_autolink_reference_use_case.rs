use kube::runtime::events::{Event, EventType, Recorder};
use kube::Resource;

use crate::domain::model::autolink_reference::AutolinkReference;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

pub struct DeleteAutolinkReferenceUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl DeleteAutolinkReferenceUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        autolink_reference: &AutolinkReference,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        if let Some(status) = &autolink_reference.status {
            if let Some(id) = &status.id {
                log::info!(
                    "delete autolink reference: {}/{}",
                    &autolink_reference.spec.full_name,
                    id
                );
                self.github_service
                    .delete_autolink_references(&autolink_reference.spec.full_name, id)
                    .await?;
                let reference = autolink_reference.object_ref(&());
                recorder
                    .publish(
                        &Event {
                            action: "autolink-reference-deleted".into(),
                            reason: "Reconciling".into(),
                            note: Some("Autolink reference deleted".into()),
                            type_: EventType::Normal,
                            secondary: None,
                        },
                        &reference,
                    )
                    .await
                    .map_err(ControllerError::KubeError)?;
            }
        }
        Ok(())
    }
}
