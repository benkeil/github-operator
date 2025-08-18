use crate::domain::model::permission::RepositoryPermission;
use crate::domain::service::github_service::GitHubService;
use crate::ControllerError;

use kube::runtime::events::{Event, EventType, Recorder};
use kube::Resource;

pub struct DeletePermissionUseCase {
    github_service: Box<dyn GitHubService + Send + Sync>,
}

impl DeletePermissionUseCase {
    pub fn new(github_service: Box<dyn GitHubService + Send + Sync>) -> Self {
        Self { github_service }
    }

    pub async fn execute(
        &self,
        permission: &RepositoryPermission,
        recorder: Recorder,
    ) -> Result<(), ControllerError> {
        log::info!("delete: {}", &permission.spec.full_name);
        self.github_service
            .delete_team_permission(&permission.spec.full_name, &permission.spec.full_team_name)
            .await?;
        let reference = permission.object_ref(&());
        recorder
            .publish(
                &Event {
                    action: "permission-deleted".into(),
                    reason: "Reconciling".into(),
                    note: Some("Permission deleted".into()),
                    type_: EventType::Normal,
                    secondary: None,
                },
                &reference,
            )
            .await
            .map_err(ControllerError::KubeError)
    }
}
