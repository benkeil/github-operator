use kube::runtime::events::{Event, EventType, Recorder};

use crate::domain::model::permission::{RepositoryPermissionResponse, RepositoryPermissionSpec};
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

pub struct ReconcilePermissionUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl ReconcilePermissionUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        spec: &RepositoryPermissionSpec,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        log::info!("reconcile: {}", &spec.full_name);

        // TODO double clone
        // enable additional settings if necessary
        let spec_permission: RepositoryPermissionResponse = spec.clone().into();

        let permission = self
            .github_service
            .get_team_permission(&spec.full_name, &spec_permission.full_team_name)
            .await?;

        match permission {
            // no permission, create
            None => {
                self.github_service
                    .update_team_permission(
                        &spec.full_name,
                        &spec_permission.full_team_name,
                        &spec.permission,
                    )
                    .await?;
                self.publish_created_event(&recorder).await?;
            }
            Some(permission) => {
                // permission exists and differ
                if spec_permission.permission != permission {
                    self.github_service
                        .update_team_permission(
                            &spec.full_name,
                            &spec_permission.full_team_name,
                            &spec.permission,
                        )
                        .await?;
                    self.publish_updated_event(&recorder).await?;
                }
            }
        }

        Ok(())
    }

    async fn publish_created_event(&self, recorder: &Recorder) -> Result<(), ControllerError> {
        recorder
            .publish(Event {
                action: "permission-created".into(),
                reason: "Reconciling".into(),
                note: Some("Permission created".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(ControllerError::KubeError)
    }

    async fn publish_updated_event(&self, recorder: &Recorder) -> Result<(), ControllerError> {
        recorder
            .publish(Event {
                action: "permission-updated".into(),
                reason: "Reconciling".into(),
                note: Some("Permission updated".into()),
                type_: EventType::Normal,
                secondary: None,
            })
            .await
            .map_err(ControllerError::KubeError)
    }
}
