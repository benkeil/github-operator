use k8s_openapi::api::core::v1::ObjectReference;
use kube::runtime::events::{Event, EventType, Recorder};
use kube::Resource;

use crate::domain::model::permission::{
    RepositoryPermission, RepositoryPermissionResponse, RepositoryPermissionSpec,
};
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
        repository_permission: &RepositoryPermission,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        log::info!("reconcile: {}", &repository_permission.spec.full_name);

        // TODO double clone
        // enable additional settings if necessary
        let spec_permission: RepositoryPermissionResponse =
            repository_permission.spec.clone().into();

        let permission = self
            .github_service
            .get_team_permission(
                &repository_permission.spec.full_name,
                &spec_permission.full_team_name,
            )
            .await?;
        let reference = repository_permission.object_ref(&());

        match permission {
            // no permission, create
            None => {
                self.github_service
                    .update_team_permission(
                        &repository_permission.spec.full_name,
                        &spec_permission.full_team_name,
                        &repository_permission.spec.permission,
                    )
                    .await?;
                self.publish_updated_event(&recorder, &reference).await?;
            }
            Some(permission) => {
                // permission exists and differ
                if spec_permission.permission != permission {
                    self.github_service
                        .update_team_permission(
                            &repository_permission.spec.full_name,
                            &spec_permission.full_team_name,
                            &repository_permission.spec.permission,
                        )
                        .await?;
                    self.publish_updated_event(&recorder, &reference).await?;
                }
            }
        }

        Ok(())
    }

    async fn publish_updated_event(
        &self,
        recorder: &Recorder,
        reference: &ObjectReference,
    ) -> Result<(), ControllerError> {
        recorder
            .publish(
                &Event {
                    action: "permission-updated".into(),
                    reason: "Reconciling".into(),
                    note: Some("Permission updated".into()),
                    type_: EventType::Normal,
                    secondary: None,
                },
                reference,
            )
            .await
            .map_err(ControllerError::KubeError)
    }
}
